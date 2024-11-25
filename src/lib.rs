pub mod c_macro;
pub mod header_file_proc;
pub mod source_file_proc;

use thiserror::Error;
#[derive(Error, Debug)]
pub enum Error {
    #[error("IoError")]
    IoError(#[from] std::io::Error),
}