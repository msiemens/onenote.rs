
#[macro_export]
#[cfg(target_arch = "wasm32")]
macro_rules! log {
	( $( $t:tt )* ) => {
		#[cfg(debug_assertions)]
		web_sys::console::log_2(&format!("OneNoteConverter: ").into(), &format!( $( $t )* ).into());
	}
}

#[macro_export]
#[cfg(target_arch = "wasm32")]
macro_rules! log_warn {
	( $( $t:tt )* ) => {
		use crate::utils::log::get_current_page;

		web_sys::console::warn_2(&format!("OneNoteConverter: ").into(), &format!( $( $t )* ).into());
	}
}

#[cfg(not(target_arch = "wasm32"))]
macro_rules! log {
	( $( $t:tt )* ) => {
		#[cfg(debug_assertions)]
		println!( $( $t )* );
	}
}

#[cfg(not(target_arch = "wasm32"))]
macro_rules! log_warn {
	( $( $t:tt )* ) => {
		println!("Warning: {}", &format!( $( $t )* ));
	}
}

pub(crate) use log;
pub(crate) use log_warn;
