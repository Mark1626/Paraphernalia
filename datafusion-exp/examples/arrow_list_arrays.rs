use std::sync::Arc;

use anyhow::Result;
use arrow::{
    array::{Array, Int32Array, ListArray},
    datatypes::{DataType, Field, Int32Type},
};

fn main() -> Result<()> {
    let _field = Field::new(
        "test",
        DataType::List(Arc::new(Field::new("element", DataType::Int32, true))),
        true,
    );

    let data = vec![
        Some(vec![Some(0), Some(1), Some(2)]),
        None,
        Some(vec![Some(3), None, Some(5)]),
        Some(vec![Some(6), Some(7)]),
    ];

    // Creating a list array
    let array = ListArray::from_iter_primitive::<Int32Type, _, _>(data);

    // Trying to convert it into varchar
    let data_str: Vec<String> = (0..array.len())
        .map(|i| {
            if array.is_null(i) {
                "[null]".into()
            } else {
                let arr_value = array.value(i);
                let ar = arr_value.as_any().downcast_ref::<Int32Array>().unwrap();

                let serialized_val: Vec<String> = (0..ar.len())
                    .map(|idx| {
                        if ar.is_null(idx) {
                            "null".into()
                        } else {
                            let v = ar.value(idx);
                            v.to_string()
                        }
                    })
                    .collect();

                format!("[{}]", serialized_val.join(","))
            }
        })
        .collect();

    eprintln!("Data {:?}", data_str);

    Ok(())
}
