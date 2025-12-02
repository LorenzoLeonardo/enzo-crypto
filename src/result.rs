pub trait Fallback {
    fn fallback(err: serde_json::Error) -> serde_json::Value;
}

pub struct DefaultFallback;

impl Fallback for DefaultFallback {
    fn fallback(err: serde_json::Error) -> serde_json::Value {
        serde_json::Value::String(err.to_string())
    }
}

pub struct GenericResult<T, E, F = DefaultFallback>(Result<T, E>, std::marker::PhantomData<F>);

impl<T, E, F> From<GenericResult<T, E, F>> for serde_json::Value
where
    T: serde::Serialize,
    E: serde::Serialize,
    F: Fallback,
{
    fn from(res: GenericResult<T, E, F>) -> Self {
        match res.0 {
            Ok(v) => serde_json::to_value(v).unwrap_or_else(F::fallback),
            Err(e) => serde_json::to_value(e).unwrap_or_else(F::fallback),
        }
    }
}

impl<T, E, F> GenericResult<T, E, F> {
    // Ok constructor
    pub fn ok(value: T) -> Self {
        Self(Ok(value), std::marker::PhantomData)
    }

    // Err constructor
    pub fn err(err: E) -> Self {
        Self(Err(err), std::marker::PhantomData)
    }
}
