use giftwrap::Wrap;

pub type MEEResult<T> = std::result::Result<T, Error>;

#[derive(Debug, Wrap)]
pub enum Error {
    Glib(glib::Error),
    Bool(glib::BoolError),
}
