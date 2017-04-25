#[inline]
pub fn stringify<E>(error: E) -> String
    where E: ::std::fmt::Debug + ::std::error::Error
{
    format!("Error: {:?}", error)
}

error_chain!{}
