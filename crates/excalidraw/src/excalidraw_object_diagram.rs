use crate::ExcalidrawBoundElement;
use crate::ExcalidrawElement;
use crate::ExcalidrawElementBase;
use crate::ExcalidrawElementId;
use crate::ExcalidrawObjectSnapshot;
use crate::ExcalidrawTextAlignment;
use crate::ExcalidrawVerticalAlignment;

const WIDTH: f64 = 620.0;
const GAP: f64 = 60.0;
const PADDING: f64 = 20.0;
const FONT: f64 = 16.0;

pub(crate) fn elements(
    title: &str,
    objects: &[ExcalidrawObjectSnapshot],
) -> eyre::Result<Vec<ExcalidrawElement>> {
    let mut result = vec![text(
        "objects-title",
        title,
        40.0,
        40.0,
        3.0 * WIDTH,
        28.0,
        "#1864ab",
    )];
    if objects.is_empty() {
        result.push(text(
            "objects-empty",
            "No objects match the current tab.",
            40.0,
            100.0,
            WIDTH,
            24.0,
            "#868e96",
        ));
        return Ok(result);
    }
    let mut y = 120.0;
    for (row, objects_row) in objects.chunks(3).enumerate() {
        let mut row_height = 0.0_f64;
        for (column, object) in objects_row.iter().enumerate() {
            let index = row * 3 + column;
            let x = 40.0 + column as f64 * (WIDTH + GAP);
            let mut body = Vec::new();
            let address_height = estimated_height(&object.address, WIDTH - 2.0 * PADDING);
            let shape_y = y + 18.0 + address_height + 6.0;
            let shape_height = estimated_height(&object.shape, WIDTH - 2.0 * PADDING);
            let mut line_y = shape_y + shape_height + 16.0;
            body.push(text(
                &format!("object-{index}-address"),
                &object.address,
                x + PADDING,
                y + 18.0,
                WIDTH - 2.0 * PADDING,
                address_height,
                "#868e96",
            ));
            body.push(text(
                &format!("object-{index}-shape"),
                &object.shape,
                x + PADDING,
                shape_y,
                WIDTH - 2.0 * PADDING,
                shape_height,
                "#1864ab",
            ));
            for (field, value) in object.fields.iter().enumerate() {
                let label = match &value.type_name {
                    Some(shape) => format!("{}: {shape}", value.name),
                    None => value.name.clone(),
                };
                let label_height = estimated_height(&label, WIDTH - 2.0 * PADDING);
                let value_height = estimated_height(&value.value, WIDTH - 2.0 * PADDING - 18.0);
                body.push(text(
                    &format!("object-{index}-field-{field}"),
                    &label,
                    x + PADDING,
                    line_y,
                    WIDTH - 2.0 * PADDING,
                    label_height,
                    "#5f3dc4",
                ));
                line_y += label_height + 4.0;
                body.push(text(
                    &format!("object-{index}-value-{field}"),
                    &value.value,
                    x + PADDING + 18.0,
                    line_y,
                    WIDTH - 2.0 * PADDING - 18.0,
                    value_height,
                    "#212529",
                ));
                line_y += value_height + 14.0;
            }
            if !object.fields_complete {
                body.push(text(
                    &format!("object-{index}-limited"),
                    "More fields exist; this observation reached its field limit.",
                    x + PADDING,
                    line_y,
                    WIDTH - 2.0 * PADDING,
                    40.0,
                    "#e67700",
                ));
                line_y += 54.0;
            }
            let height = line_y - y + PADDING;
            let mut base = ExcalidrawElementBase::new(
                ExcalidrawElementId(format!("object-{index}")),
                x,
                y,
                WIDTH,
                height,
            );
            base.background_color = "#f8f9fa".into();
            base.stroke_color = "#adb5bd".into();
            result.push(ExcalidrawElement::Rectangle { base });
            result.extend(body);
            row_height = row_height.max(height);
        }
        y += row_height + GAP;
    }
    for (index, object) in objects.iter().enumerate() {
        if let Some(plan) = &object.plan {
            let mut diagram = crate::discovery_plan_diagram::elements(plan)?;
            let height = diagram
                .iter()
                .map(|element| element.base().y + element.base().height)
                .fold(0.0_f64, f64::max);
            for element in &mut diagram {
                translate_and_namespace(element, &format!("object-{index}-"), y);
            }
            result.extend(diagram);
            y += height + GAP;
        }
    }
    Ok(result)
}

fn estimated_height(value: &str, width: f64) -> f64 {
    // Reserve room for native editor wrapping without inserting artificial line breaks.
    let chars = (width / (FONT * 0.6)).floor().max(1.0) as usize;
    value
        .lines()
        .map(|line| line.chars().count().max(1).div_ceil(chars))
        .sum::<usize>()
        .max(1) as f64
        * FONT
        * 1.25
}

fn text(
    id: &str,
    content: &str,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    color: &str,
) -> ExcalidrawElement {
    let mut base =
        ExcalidrawElementBase::new(ExcalidrawElementId(id.to_owned()), x, y, width, height);
    base.stroke_color = color.to_owned();
    ExcalidrawElement::Text {
        base,
        text: content.to_owned(),
        font_size: FONT,
        font_family: 3,
        text_align: ExcalidrawTextAlignment::Left,
        vertical_align: ExcalidrawVerticalAlignment::Top,
        container_id: None,
        original_text: content.to_owned(),
        auto_resize: false,
        line_height: 1.25,
    }
}

fn translate_and_namespace(element: &mut ExcalidrawElement, prefix: &str, y: f64) {
    let base = match element {
        ExcalidrawElement::Rectangle { base } => base,
        ExcalidrawElement::Text {
            base, container_id, ..
        } => {
            if let Some(id) = container_id {
                id.0.insert_str(0, prefix);
            }
            base
        }
        ExcalidrawElement::Arrow {
            base,
            start_binding,
            end_binding,
            ..
        } => {
            for binding in [start_binding, end_binding].into_iter().flatten() {
                binding.element_id.0.insert_str(0, prefix);
            }
            base
        }
    };
    base.y += y;
    base.id.0.insert_str(0, prefix);
    base.group_ids.clear();
    for binding in &mut base.bound_elements {
        let (ExcalidrawBoundElement::Text { id } | ExcalidrawBoundElement::Arrow { id }) = binding;
        id.0.insert_str(0, prefix);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ExcalidrawObjectField;
    use crate::ExcalidrawScene;
    use cloud_terrastodon_registry::DiscoveryPlan;
    use std::collections::BTreeSet;

    #[test]
    fn object_fields_remain_separately_editable_and_multiple_plans_have_unique_ids() {
        let object = ExcalidrawObjectSnapshot {
            address: "slot 42".into(),
            shape: "Example".into(),
            fields: vec![ExcalidrawObjectField {
                name: "name".into(),
                type_name: Some("String".into()),
                value: "a project".into(),
            }],
            fields_complete: false,
            plan: Some(DiscoveryPlan {
                nodes: vec![],
                edges: vec![],
                diagnostics: vec![],
            }),
        };
        let scene =
            ExcalidrawScene::from_objects("Current tab", &[object.clone(), object]).unwrap();
        let ids = scene
            .elements
            .iter()
            .map(|e| &e.base().id.0)
            .collect::<BTreeSet<_>>();
        assert_eq!(ids.len(), scene.elements.len());
        assert!(scene.elements.iter().all(|e| e.base().group_ids.is_empty()));
        assert!(scene.elements.iter().any(|e| matches!(e, ExcalidrawElement::Text { text, container_id: None, .. } if text == "name: String")));
        assert!(scene.elements.iter().any(
            |e| matches!(e, ExcalidrawElement::Text { text, .. } if text.contains("field limit"))
        ));
    }
}
