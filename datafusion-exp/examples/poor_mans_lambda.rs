// Lambda's aren't supported in Datafusion 51
// https://github.com/apache/datafusion/issues/14205
// This PR brings it in
// https://github.com/apache/datafusion/pull/18921/changes#diff-650e726cb026e662f89df1c592963d1b1d9df8261890ffc0b6930a212c5498ed
//
// This is a small experiment to create a UDF which can store a physical expression of the lambda
// in the function, which is then evaluated during runtime.
use std::any::Any;
use std::fmt::Debug;
use std::{fmt, sync::Arc};

use arrow::array::{Array, AsArray, GenericListArray, ListArray, RecordBatch};
use arrow::datatypes::{DataType, Field, Int32Type, Schema};
use datafusion::config::ConfigOptions;
use datafusion::error::{DataFusionError, Result};

use datafusion::logical_expr::Operator;
use datafusion::physical_plan::ColumnarValue as PhysicalColumnarValue;
use datafusion::physical_plan::expressions::{BinaryExpr, Column, Literal};
use datafusion::scalar::ScalarValue;
use datafusion::{
    logical_expr::{
        ColumnarValue, ScalarFunctionArgs, ScalarUDF, ScalarUDFImpl, Signature, Volatility,
    },
    physical_plan::PhysicalExpr,
};

#[derive(Eq, Hash)]
pub struct PoormansLambdaTransformUDF {
    signature: Signature,
    physical_expr: Arc<dyn PhysicalExpr>,
}

impl Debug for PoormansLambdaTransformUDF {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PoormansLambdaTransformUDF")
            .field("signature", &self.signature)
            .field("physical_expr", &self.physical_expr)
            .finish()
    }
}

impl PartialEq for PoormansLambdaTransformUDF {
    // There's no way to currently compare two UDFs
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl PoormansLambdaTransformUDF {
    pub fn new(physical_expr: Arc<dyn PhysicalExpr>) -> Self {
        Self {
            signature: Signature::any(1, Volatility::Immutable),
            physical_expr,
        }
    }
}

impl ScalarUDFImpl for PoormansLambdaTransformUDF {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self) -> &str {
        "list_transform"
    }

    fn signature(&self) -> &Signature {
        &self.signature
    }

    fn return_type(&self, arg_types: &[DataType]) -> Result<DataType> {
        // Input should be a List
        match &arg_types[0] {
            DataType::List(field) => {
                // The output type depends on what the physical expression returns
                // For simplicity, we'll return the same list structure
                // In practice, you'd want to infer this from the physical expression
                Ok(DataType::List(field.clone()))
            }
            _ => Err(datafusion::error::DataFusionError::Plan(
                "Expected List type".to_string(),
            )),
        }
    }

    fn invoke_with_args(
        &self,
        args: datafusion::logical_expr::ScalarFunctionArgs,
    ) -> Result<ColumnarValue> {
        let array = match &args.args[0] {
            ColumnarValue::Array(array) => array.clone(),
            ColumnarValue::Scalar(scalar) => scalar.to_array()?,
        };

        let list_array = array.as_list::<i32>();
        let value_field = match list_array.data_type() {
            DataType::List(field) => Arc::clone(field),
            _ => return Err(DataFusionError::Execution("Not a list".into())),
        };

        let schema = Arc::new(Schema::new(vec![Arc::clone(&value_field)]));
        let list_values = Arc::clone(&list_array.values());
        let elem_length = list_values.len();

        let rb = RecordBatch::try_new(Arc::clone(&schema), vec![list_values])?;
        let results = self.physical_expr.evaluate(&rb)?;

        let result_array = match results {
            PhysicalColumnarValue::Array(arr) => arr,
            PhysicalColumnarValue::Scalar(scalar) => scalar.to_array_of_size(elem_length)?,
        };

        let gla = GenericListArray::try_new(
            Arc::new(Field::new_list_field(
                value_field.data_type().clone(),
                value_field.is_nullable(),
            )),
            list_array.offsets().clone(),
            Arc::new(result_array),
            list_array.nulls().cloned(),
        )?;

        Ok(ColumnarValue::Array(Arc::new(gla)))
    }
}

fn main() -> anyhow::Result<()> {
    // let lambda = Arc::new(Column::new("test", 0));
    let col_expr = Arc::new(Column::new("test", 0));
    let literal_expr = Arc::new(Literal::new(ScalarValue::Int32(Some(2))));
    let lambda = Arc::new(BinaryExpr::new(col_expr, Operator::Plus, literal_expr));

    let list_type = Arc::new(Field::new_list_field(DataType::Int32, true));
    let field = Arc::new(Field::new(
        "test",
        DataType::List(Arc::clone(&list_type)),
        true,
    ));

    let return_type = Arc::clone(&field);

    let data = vec![
        Some(vec![Some(0), Some(1), Some(2)]),
        None,
        Some(vec![Some(3), None, Some(5)]),
        Some(vec![Some(6), Some(7)]),
    ];

    // Creating a list array
    let array = ColumnarValue::Array(Arc::new(ListArray::from_iter_primitive::<Int32Type, _, _>(
        data,
    )));

    eprintln!("Lambda Expr {:?}", lambda);

    let udf_impl = PoormansLambdaTransformUDF::new(lambda);
    let udf = ScalarUDF::from(udf_impl);

    let config_options = Arc::new(ConfigOptions::default());

    eprintln!("Input \n {:}", array);

    let scalar_func_args = ScalarFunctionArgs {
        args: vec![array],
        arg_fields: vec![field],
        number_rows: 1,
        return_field: return_type,
        config_options,
    };

    let r = udf.invoke_with_args(scalar_func_args)?;
    eprintln!("Result \n {:}", r);

    Ok(())
}
