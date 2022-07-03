extern crate proc_macro;

use proc_macro::TokenStream;

use syn::{parse_macro_input, ExprClosure};
use quote::quote;

#[proc_macro]
pub fn app_data(input: TokenStream) -> TokenStream {
    let parsed_input = parse_macro_input!(input as ExprClosure);
    let closure_input = parsed_input.inputs;
    let closure_body = parsed_input.body;

    let window_parse = quote! {
        // Base window
        if win.transient_for().is_none() {
            let application = win.application().unwrap();
            let beedget_application = application.downcast_ref::<crate::BeedgetApplication>().unwrap();
            let data = beedget_application.imp().data.get().unwrap();

            closure(data);
        }
        // Dialog window
        else {
            let window = win.transient_for().unwrap();
            let application = window.application().unwrap();
            let beedget_application = application.downcast_ref::<crate::BeedgetApplication>().unwrap();
            let data = beedget_application.imp().data.get().unwrap();

            closure(data);
        }
    };

    let widget_parse = quote! {
        let mut window = self.parent().unwrap();

        while window.downcast_ref::<gtk::Window>().is_none() {
            window = window.parent().unwrap();
        }

        let window = window.downcast_ref::<gtk::Window>().unwrap();
        let application = window.application().unwrap();
        let beedget_application = application.downcast_ref::<crate::BeedgetApplication>().unwrap();
        let data = beedget_application.imp().data.get().unwrap();

        closure(data);
    };

    let ret = quote! {
        let closure = |#closure_input: &crate::models::SaveData| #closure_body;

        if let Some(win) = self.dynamic_cast_ref::<gtk::Window>() {
            #window_parse
        }
        else {
            #widget_parse
        }
    };

    TokenStream::from(ret)
}
