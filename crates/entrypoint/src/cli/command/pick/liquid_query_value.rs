use eyre::Result;
use eyre::WrapErr;
use eyre::bail;
use facet_value::DestructuredRef;
use facet_value::Value;
use liquid::Object;
use liquid::model::KString;
use liquid::model::Value as LiquidValue;

/// Convert the picker JSON into the Liquid engine's native data model.
pub(super) fn liquid_query_value(value: &Value) -> Result<LiquidValue> {
    Ok(match value.destructure_ref() {
        DestructuredRef::Null => LiquidValue::Nil,
        DestructuredRef::Bool(value) => LiquidValue::scalar(value),
        DestructuredRef::Number(value) if value.is_float() => {
            LiquidValue::scalar(value.to_f64_lossy())
        }
        DestructuredRef::Number(value) => LiquidValue::scalar(
            value
                .to_i64()
                .ok_or_else(|| eyre::eyre!("number is out of range for a Liquid integer"))?,
        ),
        DestructuredRef::String(value) => LiquidValue::scalar(KString::from_ref(value.as_str())),
        DestructuredRef::Array(values) => {
            let values = values
                .iter()
                .map(liquid_query_value)
                .collect::<Result<Vec<_>>>()?;
            LiquidValue::Array(values)
        }
        DestructuredRef::Object(values) => {
            let mut object = Object::new();
            for (name, value) in values {
                let value = liquid_query_value(value)
                    .wrap_err_with(|| format!("converting Liquid field {name}"))?;
                object.insert(KString::from_ref(name.as_str()), value);
            }
            LiquidValue::Object(object)
        }
        _ => {
            bail!("Liquid query input contains a non-JSON value");
        }
    })
}
