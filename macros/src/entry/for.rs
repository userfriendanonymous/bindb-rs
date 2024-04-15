use proc_macro::TokenStream;
use quote::quote;


pub fn derive(item: syn::ItemType, meta: super::input::Meta, lib: syn::Path) -> TokenStream {
    let item_ty = item.ty;
    let item_generics = item.generics;

    let len = *meta.len.unwrap().expr;

    let buf_info = meta.buf.unwrap();
    let buf_ident = match buf_info.ty.as_ref() {
        syn::Type::Path(path) => path.path.require_ident().unwrap(),
        _ => panic!("Type must be an indent.")
    };
    let buf_vis = buf_info.vis.clone();
    let mut buf_generics = buf_info.generics;
    let buf_generics_params = buf_generics.params;
    let (_, buf_ty_generics, buf_where_clause) = buf_generics.split_for_impl();
    // let buf_ty_generics_params = buf_ty_generics.

    quote! {
        #buf_vis struct #buf_ident<BV: #lib::entry::bytes::Variant, buf_generics_params>(#lib::entry::Bytes<BV, { #len }>, PhantomData) #buf_where_clause;

        impl #item_generics #lib::Entry for #item_ty {
            const LEN: usize = #len;
            type Buf<BV: #lib::entry::bytes::Variant> = #buf_ident<BV>;

            fn buf<BV: #lib::entry::bytes::Variant>(bytes: #lib::entry::Bytes<BV, { #len }>) -> Self::Buf<BV> {
                #buf_ident(bytes)
            }
            fn buf_mut_as_const<'a>(buf: &'a #lib::entry::BufMut<'a, Self>) -> #lib::entry::BufConst<'a, Self> {
                #buf_ident(buf.0.as_const())
            }
            fn buf_owned_as_const(buf: &#lib::entry::BufOwned<Self>) -> #lib::entry::BufConst<'_, Self> {
                #buf_ident(buf.0.as_const())
            }
            fn buf_owned_as_mut(buf: &mut #lib::entry::BufOwned<Self>) -> #lib::entry::BufMut<'_, Self> {
                #buf_ident(buf.0.as_mut())
            }
            fn buf_rb_const<'a>(buf: &'a #lib::entry::BufConst<'a, Self>) -> #lib::entry::BufConst<'a, Self> {
                #buf_ident(buf.0.rb_const())
            }
            fn buf_rb_mut<'a>(buf: &'a mut #lib::entry::BufMut<'a, Self>) -> #lib::entry::BufMut<'a, Self> {
                #buf_ident(buf.0.rb_mut())
            }
            unsafe fn buf_detach<'b, BV: #lib::entry::bytes::variant::Ref>(buf: Self::Buf<BV>) -> Self::Buf<BV::Ref<'b>> {
                #buf_ident(buf.0.detach())
            }
            fn buf_copy_to(src: #lib::entry::BufConst<'_, Self>, mut dst: #lib::entry::BufMut<'_, Self>) {
                dst.0.copy_from(&src.0)
            }
            fn buf_swap(mut a: #lib::entry::BufMut<'_, Self>, mut b: #lib::entry::BufMut<'_, Self>) {
                a.0.swap(&mut b.0)
            }
        }
    }.into()
}
