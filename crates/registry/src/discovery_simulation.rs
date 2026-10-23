use crate::CancellationScope;
use crate::DiscoveryValue;
use crate::Function;
use crate::FunctionExecutor;
use crate::FunctionKind;
use crate::RuntimeValue;
use crate::describe_function;
use crate::describe_shape;
use crate::discovery_arbitrary::fabricate_response;
use crate::discovery_input::assemble_request;
use crate::discovery_reflection::reflected_descendants;
use crate::known_thing_for_shape;
use facet::Shape;
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ptr;

/// Offline exploration of registered type coverage from explicit seed values.
///
/// Each actual Facet shape retains one representative. This is a bounded
/// coverage approximation, not a production dump scheduler: it does not prove
/// that values from different organizations or tenants may be combined.
/// Convergence means this sampled pool reached a fixed point, not that all
/// possible Arbitrary payloads or live service objects have been explored.
///
/// Request executors are never called. Audited reads without solicitation
/// contribute only fabricated response values, modeling unattended discovery.
/// The only real operations invoked are declared
/// pure synchronous conversions/projections and Arbitrary response constructors
/// in the supplied catalog. The catalog must include every required generator;
/// linked functions outside it are never used as a fallback. Callers must audit
/// the supplied operations and generators.
pub struct DiscoverySimulation {
    /// Values indexed by their actual Facet shape. Discovery currently keeps the
    /// first representative in each bucket; map iteration order is unspecified.
    pub values: HashMap<&'static Shape, Vec<DiscoveryValue>>,
    /// Admitted request edges, including those whose response generation failed.
    /// Inspect diagnostics and retained values when assessing actual coverage.
    pub simulated_requests: Vec<&'static Function>,
    pub rounds: usize,
    pub converged: bool,
    pub diagnostics: Vec<String>,
}

impl DiscoverySimulation {
    /// Explore until the representative type pool stops growing or a caller
    /// budget is exhausted. Missing request inputs are never fabricated or
    /// defaulted into domain identities; optional request fields may be None.
    /// Pass a borrowed registry slice or an iterable of static function handles.
    /// Its iterator must support cloning to replay traversal in each round.
    ///
    /// `max_values` caps entries in the representative pool, including seeds,
    /// assembled requests, responses, and cloned descendants. It does not cap
    /// elements or bytes already owned inside a retained container value.
    pub fn run<I>(
        seeds: Vec<RuntimeValue>,
        functions: I,
        seed: u64,
        max_rounds: usize,
        max_values: usize,
        cancellation: &CancellationScope,
    ) -> eyre::Result<Self>
    where
        I: IntoIterator<Item = &'static Function>,
        I::IntoIter: Clone,
    {
        let functions = functions.into_iter();
        let mut simulation = Self {
            values: HashMap::new(),
            simulated_requests: Vec::new(),
            rounds: 0,
            converged: false,
            diagnostics: Vec::new(),
        };
        for value in seeds {
            cancellation.bail_if_cancelled()?;
            if !simulation.retain_value(
                DiscoveryValue {
                    value,
                    depth: 0,
                    producer: None,
                },
                max_values,
                cancellation,
            )? {
                return Ok(simulation);
            }
        }
        let mut random = StdRng::seed_from_u64(seed);
        // Attempt each traversal registration once after its input is available,
        // even if the attempt fails. Metadata addresses identify registrations:
        // separate entries may share a signature or executor. Missing-input
        // operations remain outside this set so later rounds can reconsider them.
        // Traversal follows catalog order rather than set iteration.
        let mut completed = HashSet::<*const Function>::new();
        let mut blocked = Vec::<(&'static Function, String)>::new();
        for round in 0..max_rounds {
            cancellation.bail_if_cancelled()?;
            simulation.rounds = round + 1;
            let previous_shape_count = simulation.values.len();
            let previous_completed = completed.len();
            blocked.clear();
            for function in functions.clone() {
                cancellation.bail_if_cancelled()?;
                let registration = ptr::from_ref(function);
                if completed.contains(&registration) || !eligible(function) {
                    continue;
                }
                if function.kind == FunctionKind::AsyncInvoke {
                    let depth = if let Some(request) = simulation
                        .values
                        .get(function.input_shape)
                        .and_then(|values| values.first())
                    {
                        // Explicit seeds are already complete input contracts.
                        // Inspecting them needs neither Clone nor reconstructing
                        // their filters from unrelated representative fields.
                        request.depth
                    } else {
                        let request = assemble_request(
                            function.input_shape,
                            &simulation.values,
                            cancellation,
                        );
                        cancellation.bail_if_cancelled()?;
                        let (request, depth) = match request {
                            Ok(request) => request,
                            Err(error) => {
                                blocked.push((function, format!("{error:#}")));
                                continue;
                            }
                        };
                        if !simulation.retain_value(
                            DiscoveryValue {
                                value: request,
                                depth,
                                producer: None,
                            },
                            max_values,
                            cancellation,
                        )? {
                            return Ok(simulation);
                        }
                        depth
                    };
                    // Merely inspecting the request is enough: its executor and
                    // cancellation-binding callback remain unreachable here.
                    simulation.simulated_requests.push(function);
                    completed.insert(registration);
                    if simulation.values.contains_key(function.output_shape) {
                        continue;
                    }
                    let response = fabricate_response(
                        function.output_shape,
                        functions.clone(),
                        &mut random,
                        cancellation,
                        &mut simulation.diagnostics,
                    );
                    cancellation.bail_if_cancelled()?;
                    let response = match response {
                        Ok(response) => response,
                        Err(error) => {
                            simulation.diagnostics.push(format!(
                                "Cannot simulate {}: {error:#}",
                                describe_function(function)
                            ));
                            continue;
                        }
                    };
                    if !simulation.retain_value(
                        DiscoveryValue {
                            value: response,
                            depth: depth.saturating_add(1),
                            producer: Some(function),
                        },
                        max_values,
                        cancellation,
                    )? {
                        return Ok(simulation);
                    }
                } else {
                    let Some(input) = simulation
                        .values
                        .get(function.input_shape)
                        .and_then(|values| values.first())
                    else {
                        blocked.push((
                            function,
                            format!(
                                "no discovered value of {}",
                                describe_shape(function.input_shape)
                            ),
                        ));
                        continue;
                    };
                    let depth = input.depth.saturating_add(1);
                    let output = invoke_pure(function, &input.value);
                    cancellation.bail_if_cancelled()?;
                    completed.insert(registration);
                    match output {
                        Ok(value) => {
                            if !simulation.retain_value(
                                DiscoveryValue {
                                    value,
                                    depth,
                                    producer: Some(function),
                                },
                                max_values,
                                cancellation,
                            )? {
                                return Ok(simulation);
                            }
                        }
                        Err(error) => simulation.diagnostics.push(format!(
                            "Cannot apply {}: {error:#}",
                            describe_function(function)
                        )),
                    }
                }
            }
            if previous_shape_count == simulation.values.len()
                && previous_completed == completed.len()
            {
                simulation.converged = true;
                break;
            }
        }
        for (function, reason) in blocked {
            simulation
                .diagnostics
                .push(format!("Blocked {}: {reason}", describe_function(function)));
        }
        if !simulation.converged {
            simulation.diagnostics.push(format!(
                "Stopped at the {max_rounds}-round simulation budget before convergence"
            ));
        }
        Ok(simulation)
    }

    /// Keep the first representative of each shape. The boolean tells the caller
    /// whether exploration may continue, not whether this candidate was inserted.
    /// A budget stop can occur after retaining the candidate and some descendants.
    fn retain_value(
        &mut self,
        value: DiscoveryValue,
        max_values: usize,
        cancellation: &CancellationScope,
    ) -> eyre::Result<bool> {
        if self.values.contains_key(value.value.shape()) {
            // The first representative wins. Discard this candidate without
            // inserting it or reflecting its alternative payload: the pool's
            // length is unchanged, so a duplicate spends no value-budget slot.
            return Ok(true);
        }
        let retained_count = self.values.values().map(Vec::len).sum::<usize>();
        if retained_count >= max_values {
            self.diagnostics.push(format!(
                "Stopped at the {max_values}-value simulation budget before convergence"
            ));
            return Ok(false);
        }
        // Admission above guarantees a free slot. Reserve it for this parent;
        // reflection may retain only the slots left after that reservation, so
        // parent plus descendants cannot grow the pool beyond max_values.
        let (descendants, reflection_complete) = reflected_descendants(
            &value.value,
            value.depth,
            value.producer,
            &self.values,
            max_values - retained_count - 1,
            cancellation,
            &mut self.diagnostics,
        )?;
        let retained_count = retained_count + 1 + descendants.len();
        self.values
            .entry(value.value.shape())
            .or_default()
            .push(value);
        for descendant in descendants {
            self.values
                .entry(descendant.value.shape())
                .or_default()
                .push(descendant);
        }
        if !reflection_complete {
            return Ok(false);
        }
        if retained_count >= max_values {
            // Stop as soon as the last slot is filled. We cannot claim convergence
            // without exploring whether later work would introduce another shape.
            self.diagnostics.push(format!(
                "Stopped at the {max_values}-value simulation budget before convergence"
            ));
            return Ok(false);
        }
        Ok(true)
    }
}

fn eligible(function: &Function) -> bool {
    match function.kind {
        FunctionKind::AsyncInvoke => function.is_read_only() && !function.requires_user_input(),
        FunctionKind::Conversion | FunctionKind::Projection => {
            function.effects.is_empty() && !function.is_async()
        }
        FunctionKind::Constructor => false,
    }
}

fn invoke_pure(function: &Function, input: &RuntimeValue) -> eyre::Result<RuntimeValue> {
    let thing = known_thing_for_shape(function.input_shape).ok_or_else(|| {
        eyre::eyre!(
            "{} is not a registered input thing",
            describe_shape(function.input_shape)
        )
    })?;
    let mut input = thing.runtime_into_boxed(input.try_clone()?)?;
    // Calling these synchronous function pointers directly structurally excludes
    // async invocation and any executable-request cancellation binding callback.
    let output = match function.executor {
        FunctionExecutor::SyncValue(invoke) => invoke(input),
        FunctionExecutor::SyncRef(invoke) => invoke(input.as_ref()),
        FunctionExecutor::SyncMut(invoke) => invoke(input.as_mut()),
        FunctionExecutor::AsyncValue(_) => {
            eyre::bail!("async conversion is ineligible");
        }
    }?;
    (function.output_to_runtime)(output)
}
