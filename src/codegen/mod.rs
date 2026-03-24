pub mod html;
pub mod css;
pub mod js;
pub mod ssg;

pub use html::generate_html;
pub use css::generate_css;
pub use js::JsCodegen;
pub use ssg::render_page_html;
