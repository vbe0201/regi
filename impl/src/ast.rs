use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    Path, Token,
};

pub struct Input<P: Parse> {
    pub krate: Option<Path>,
    pub item: P,
}

pub struct RegisterBlock {
    pub attrs: Vec<syn::Attribute>,
    pub vis: syn::Visibility,
    pub ident: syn::Ident,
    pub registers: Punctuated<RegisterLayout, Token![,]>,
}

pub struct RegisterLayout {
    pub attrs: Vec<syn::Attribute>,
    pub addr: syn::LitInt,
    pub reg: RegisterDef,
}

pub struct RegisterDef {
    pub vis: syn::Visibility,
    pub ident: syn::Ident,
    pub ty: syn::Type,
    pub fields: Punctuated<BitField, Token![,]>,
}

pub struct BitField {
    pub permission: Permission,
    pub ident: syn::Ident,
    pub range: RegisterRange,
    pub options: Option<FieldOptions>,
}

pub struct FieldOptions {
    pub ident: syn::Ident,
    pub discriminants: Punctuated<(syn::Ident, syn::Expr), Token![,]>,
}

pub enum RegisterRange {
    Lit(syn::LitInt),
    Range(syn::ExprRange),
}

pub enum Permission {
    Read,
    Write,
    ReadWrite,
}

impl<P: Parse> Parse for Input<P> {
    fn parse(input: ParseStream) -> Result<Self> {
        let krate = if input.peek(Token![#]) && input.peek2(Token![!]) {
            // #![crate = regi]

            input.parse::<Token![#]>()?;
            input.parse::<Token![!]>()?;

            let content;
            syn::bracketed!(content in input);

            content.parse::<Token![crate]>()?;
            content.parse::<Token![=]>()?;

            Some(content.parse()?)
        } else {
            None
        };

        Ok(Self {
            krate,
            item: input.parse()?,
        })
    }
}

impl Parse for RegisterBlock {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let vis = input.parse()?;
        let ident = input.parse()?;

        let content;
        braced!(content in input);
        let registers = content.parse_terminated(RegisterLayout::parse)?;

        Ok(Self {
            attrs,
            vis,
            ident,
            registers,
        })
    }
}

impl Parse for RegisterLayout {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;

        let addr = input.parse()?;
        input.parse::<Token![=]>()?;
        input.parse::<Token![>]>()?;
        let reg = input.parse()?;

        Ok(Self { attrs, addr, reg })
    }
}

impl Parse for RegisterDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let vis = input.parse()?;
        let ident = input.parse()?;

        input.parse::<Token![as]>()?;
        let ty = input.parse()?;

        let content;
        braced!(content in input);
        let fields = content.parse_terminated(BitField::parse)?;

        Ok(Self {
            vis,
            ident,
            ty,
            fields,
        })
    }
}

impl Parse for BitField {
    fn parse(input: ParseStream) -> Result<Self> {
        let permission = input.parse()?;

        let ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let range = input.parse()?;

        let options = if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self {
            permission,
            ident,
            range,
            options,
        })
    }
}

impl Parse for FieldOptions {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Token![enum]>()?;

        let ident = input.parse()?;
        let content;
        braced!(content in input);

        Ok(Self {
            ident,
            discriminants: content.parse_terminated(|buf| Ok((buf.parse()?, buf.parse()?)))?,
        })
    }
}

impl Parse for RegisterRange {
    fn parse(input: ParseStream) -> Result<Self> {
        match input.parse::<syn::Expr>()? {
            syn::Expr::Lit(lit) => match lit.lit {
                syn::Lit::Int(int) => Ok(RegisterRange::Lit(int)),
                lit => Err(syn::Error::new_spanned(lit, "expected an integer literal")),
            },
            syn::Expr::Range(range) => Ok(RegisterRange::Range(range)),
            expr => Err(syn::Error::new_spanned(
                expr,
                "expected a numeric literal or a range expression denoting the field width",
            )),
        }
    }
}

impl Parse for Permission {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse::<syn::Ident>()?;

        match ident.to_string().as_str() {
            "r" => Ok(Permission::Read),
            "w" => Ok(Permission::Write),
            "rw" => Ok(Permission::ReadWrite),
            _ => Err(syn::Error::new_spanned(
                ident,
                "unrecognized permissions - r/w/rw are supported values",
            )),
        }
    }
}
