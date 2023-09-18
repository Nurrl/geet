use crate::repository::authority;

#[derive(Debug)]
pub enum Error<E> {
    Err(E),
    Warn(E),
}

impl<E> From<authority::Error> for Error<E>
where
    E: From<authority::Error>,
{
    fn from(value: authority::Error) -> Self {
        Self::Err(value.into())
    }
}

impl<E> From<git2::Error> for Error<E>
where
    E: From<git2::Error>,
{
    fn from(value: git2::Error) -> Self {
        Self::Err(value.into())
    }
}

impl<E> From<std::io::Error> for Error<E>
where
    E: From<std::io::Error>,
{
    fn from(value: std::io::Error) -> Self {
        Self::Err(value.into())
    }
}
