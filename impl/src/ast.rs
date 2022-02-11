use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    Path, Token,
};

/// Generic input type for every proc macro from this crate.
pub struct Input<P: Parse> {
    pub krate: Option<Path>,
    pub item: P,
}

/// A register block structure that defines several [registers][RegisterLayout]
/// and their mapping offsets in memory.
pub struct RegisterBlock {
    pub attrs: Vec<syn::Attribute>,
    pub vis: syn::Visibility,
    pub ident: syn::Ident,
    pub registers: Punctuated<RegisterLayout, Token![,]>,
}

/// The memory layout of a register as part of a [`RegisterBlock`] struct.
///
/// This struct encodes information on the [`RegisterDef`] and the relative
/// offset from a base address where it is mapped in memory.
pub struct RegisterLayout {
    pub attrs: Vec<syn::Attribute>,
    pub addr: syn::LitInt,
    pub reg: RegisterDef,
}

impl RegisterLayout {
    /// Gets the address offset of the register in memory.
    pub fn address(&self) -> Result<usize> {
        self.addr.base10_parse()
    }
}

/// A register definition with its name, type and several [`BitField`]s.
pub struct RegisterDef {
    pub attrs: Vec<syn::Attribute>,
    pub vis: syn::Visibility,
    pub ident: syn::Ident,
    pub ty: syn::Type,
    pub fields: Punctuated<BitField, Token![,]>,
}

/// An individual bit field definition within a [register][RegisterDef].
pub struct BitField {
    pub attrs: Vec<syn::Attribute>,
    pub permission: Permission,
    pub ident: syn::Ident,
    pub range: RegisterRange,
    pub options: Option<FieldOptions>,
}

pub struct FieldOptions {
    pub ident: syn::Ident,
    pub discriminants: Punctuated<(syn::Ident, syn::Expr), Token![,]>,
}

/// The bit range of a register field.
pub enum RegisterRange {
    Lit(syn::LitInt),
    Range(syn::ExprRange),
}

impl RegisterRange {
    fn extract_int_from_range(range: &Option<Box<syn::Expr>>) -> Result<Option<usize>> {
        match range {
            Some(expr) => match &**expr {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Int(int),
                    ..
                }) => int.base10_parse().map(Some),
                expr => Err(syn::Error::new_spanned(expr, "expected an integer literal")),
            },
            None => Ok(None),
        }
    }

    /// Attempts to get the starting bit of the range expression.
    pub fn start(&self) -> Result<usize> {
        match self {
            RegisterRange::Lit(lit) => lit.base10_parse(),
            RegisterRange::Range(range) => {
                Self::extract_int_from_range(&range.from).map(|i| i.unwrap_or(0))
            }
        }
    }

    /// Attempts to get the bit width of the range expression.
    ///
    /// The width includes the starting bit obtained with
    /// [`RegisterRange::start`].
    ///
    /// When the value is none, it means that the range spans the entire
    /// remaining bit width of the register and should be accordingly
    /// processed.
    pub fn end(&self) -> Result<Option<usize>> {
        match self {
            RegisterRange::Lit(_) => Ok(Some(1)),
            RegisterRange::Range(range) => {
                let start = self.start()?;
                let end = Self::extract_int_from_range(&range.to)?;

                if let Some(end) = end {
                    if start < end {
                        return Err(syn::Error::new_spanned(
                            &range,
                            "end of range must not be smaller than start of range",
                        ));
                    }
                }

                Ok(end
                    .map(|i| {
                        // Fix the range value by adding `1` when the end is inclusive.
                        let inclusive_end = matches!(range.limits, syn::RangeLimits::Closed(_));
                        i + inclusive_end as usize
                    })
                    .map(|i| i - start))
            }
        }
    }
}

/// The permissions levels for register bitfield access.
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
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let vis = input.parse()?;
        let ident = input.parse()?;

        input.parse::<Token![as]>()?;
        let ty = input.parse()?;

        let content;
        braced!(content in input);
        let fields = content.parse_terminated(BitField::parse)?;

        Ok(Self {
            attrs,
            vis,
            ident,
            ty,
            fields,
        })
    }
}

impl Parse for BitField {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
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
            attrs,
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
