
pub mod header_file;
pub mod c_macro;

use thiserror::Error;
#[derive(Error, Debug)]
pub enum Error {
    #[error("IoError")]
    IoError(#[from] std::io::Error),
}