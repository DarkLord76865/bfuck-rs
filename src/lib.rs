pub mod code;
pub mod compile;
pub mod error;
pub mod interpret;
pub mod io;
pub mod jit;
pub mod text;



#[doc(inline)]
pub use code::process_code;

#[doc(inline)]
pub use error::Error;

#[doc(inline)]
pub use interpret::interpret;

#[doc(inline)]
pub use jit::jit;
