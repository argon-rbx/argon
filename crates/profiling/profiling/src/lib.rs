pub use ::profiling_procmacros::function;

#[macro_export]
macro_rules! scope {
	($name:expr) => {
		puffin::profile_scope!($name);
	};
	($name:expr, $data:expr) => {
		puffin::profile_scope!($name, $data);
	};
}

#[macro_export]
macro_rules! start_frame {
	() => {
		puffin::GlobalProfiler::lock().new_frame();
	};
}
