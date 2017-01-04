#[macro_export]
macro_rules! try_io_opt {
    ($e:expr) => (match $e {
        Ok(Some(v)) => v,
        Ok(None) => return Ok(None),
        Err(e) => return Err(e)
    })
}
