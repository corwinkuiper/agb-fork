use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    Expr, Ident, LitInt, LitStr, Result, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

// -- AST types ---------------------------------------------------------------

struct UiDef {
    crate_path: TokenStream,
    name: Ident,
    ui_tiles: Expr,
    font: Expr,
    bake_settings: Expr,
    rects: Vec<RectDef>,
}

struct RectDef {
    x: LitInt,
    y: LitInt,
    w: LitInt,
    h: LitInt,
    children: Vec<ContainerChild>,
}

enum ContainerChild {
    Row(RowDef),
    Column(ColumnDef),
    Element(ElementDef),
}

struct RowDef {
    coords: Option<(LitInt, LitInt)>,
    children: Vec<ContainerChild>,
}

struct ColumnDef {
    coords: Option<(LitInt, LitInt)>,
    children: Vec<ContainerChild>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Align {
    Left,
    Right,
}

enum ElementDef {
    StaticText {
        text: LitStr,
    },
    StaticImage {
        expr: Expr,
    },
    Number {
        name: Ident,
        max_digits: LitInt,
        align: Align,
    },
    DynamicText {
        name: Ident,
        w: LitInt,
        h: LitInt,
        bake_settings: Option<Expr>,
    },
    DynamicImage {
        name: Ident,
        w: LitInt,
        h: LitInt,
    },
}

// -- Parsing -----------------------------------------------------------------

impl Parse for UiDef {
    fn parse(input: ParseStream) -> Result<Self> {
        // First token is the $crate path injected by the wrapper macro
        let crate_path: syn::Path = input.parse()?;
        let crate_path = quote! { #crate_path };
        input.parse::<Token![,]>()?;

        let name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        // ui_tiles: <expr>,
        let key: Ident = input.parse()?;
        assert_ident(&key, "ui_tiles")?;
        input.parse::<Token![:]>()?;
        let ui_tiles: Expr = input.parse()?;
        input.parse::<Token![,]>()?;

        // font: <expr>,
        let key: Ident = input.parse()?;
        assert_ident(&key, "font")?;
        input.parse::<Token![:]>()?;
        let font: Expr = input.parse()?;
        input.parse::<Token![,]>()?;

        // bake_settings: <expr>,
        let key: Ident = input.parse()?;
        assert_ident(&key, "bake_settings")?;
        input.parse::<Token![:]>()?;
        let bake_settings: Expr = input.parse()?;
        input.parse::<Token![,]>()?;

        // layout: { ... }
        let key: Ident = input.parse()?;
        assert_ident(&key, "layout")?;
        input.parse::<Token![:]>()?;
        let layout_content;
        syn::braced!(layout_content in input);

        let mut rects = Vec::new();
        while !layout_content.is_empty() {
            rects.push(parse_rect(&layout_content)?);
        }

        Ok(UiDef {
            crate_path,
            name,
            ui_tiles,
            font,
            bake_settings,
            rects,
        })
    }
}

fn assert_ident(ident: &Ident, expected: &str) -> Result<()> {
    if ident != expected {
        Err(syn::Error::new(
            ident.span(),
            format!("expected `{expected}`, found `{ident}`"),
        ))
    } else {
        Ok(())
    }
}

fn parse_rect(input: ParseStream) -> Result<RectDef> {
    let key: Ident = input.parse()?;
    assert_ident(&key, "rect")?;

    let args;
    syn::parenthesized!(args in input);
    let params: Punctuated<LitInt, Token![,]> = Punctuated::parse_separated_nonempty(&args)?;
    let params: Vec<_> = params.into_iter().collect();
    if params.len() != 4 {
        return Err(syn::Error::new(
            key.span(),
            "rect expects 4 arguments: (x, y, w, h)",
        ));
    }

    let body;
    syn::braced!(body in input);

    let children = parse_children(&body)?;

    Ok(RectDef {
        x: params[0].clone(),
        y: params[1].clone(),
        w: params[2].clone(),
        h: params[3].clone(),
        children,
    })
}

fn parse_children(input: ParseStream) -> Result<Vec<ContainerChild>> {
    let mut children = Vec::new();
    while !input.is_empty() {
        children.push(parse_container_child(input)?);
    }
    Ok(children)
}

fn parse_container_child(input: ParseStream) -> Result<ContainerChild> {
    let ident: Ident = input.parse()?;
    match ident.to_string().as_str() {
        "row" => Ok(ContainerChild::Row(parse_row_def(input)?)),
        "column" => Ok(ContainerChild::Column(parse_column_def(input)?)),
        "static_text" => Ok(ContainerChild::Element(parse_static_text(input)?)),
        "static_image" => Ok(ContainerChild::Element(parse_static_image(input)?)),
        "number" => Ok(ContainerChild::Element(parse_number(input)?)),
        "dynamic_text" => Ok(ContainerChild::Element(parse_dynamic_text(input)?)),
        "dynamic_image" => Ok(ContainerChild::Element(parse_dynamic_image(input)?)),
        _ => Err(syn::Error::new(
            ident.span(),
            format!(
                "unexpected `{ident}`, expected row, column, static_text, \
                 static_image, number, dynamic_text, or dynamic_image"
            ),
        )),
    }
}

fn parse_row_def(input: ParseStream) -> Result<RowDef> {
    let coords = if input.peek(syn::token::Paren) {
        let args;
        syn::parenthesized!(args in input);
        let params: Punctuated<LitInt, Token![,]> = Punctuated::parse_separated_nonempty(&args)?;
        let params: Vec<_> = params.into_iter().collect();
        if params.len() != 2 {
            return Err(syn::Error::new(
                Span::call_site(),
                "row coords expect 2 arguments: (x, y)",
            ));
        }
        Some((params[0].clone(), params[1].clone()))
    } else {
        None
    };

    let body;
    syn::braced!(body in input);
    let children = parse_children(&body)?;

    Ok(RowDef { coords, children })
}

fn parse_column_def(input: ParseStream) -> Result<ColumnDef> {
    let coords = if input.peek(syn::token::Paren) {
        let args;
        syn::parenthesized!(args in input);
        let params: Punctuated<LitInt, Token![,]> = Punctuated::parse_separated_nonempty(&args)?;
        let params: Vec<_> = params.into_iter().collect();
        if params.len() != 2 {
            return Err(syn::Error::new(
                Span::call_site(),
                "column coords expect 2 arguments: (x, y)",
            ));
        }
        Some((params[0].clone(), params[1].clone()))
    } else {
        None
    };

    let body;
    syn::braced!(body in input);
    let children = parse_children(&body)?;

    Ok(ColumnDef { coords, children })
}

fn parse_static_text(input: ParseStream) -> Result<ElementDef> {
    let args;
    syn::parenthesized!(args in input);
    let text: LitStr = args.parse()?;
    // consume trailing comma if present
    input.parse::<Token![,]>().ok();
    Ok(ElementDef::StaticText { text })
}

fn parse_static_image(input: ParseStream) -> Result<ElementDef> {
    let args;
    syn::parenthesized!(args in input);
    let expr: Expr = args.parse()?;
    input.parse::<Token![,]>().ok();
    Ok(ElementDef::StaticImage { expr })
}

fn parse_number(input: ParseStream) -> Result<ElementDef> {
    let name: Ident = input.parse()?;
    let args;
    syn::parenthesized!(args in input);
    let max_digits: LitInt = args.parse()?;
    args.parse::<Token![,]>()?;
    let align_ident: Ident = args.parse()?;
    let align = match align_ident.to_string().as_str() {
        "left" => Align::Left,
        "right" => Align::Right,
        _ => {
            return Err(syn::Error::new(
                align_ident.span(),
                "expected `left` or `right`",
            ))
        }
    };
    input.parse::<Token![,]>().ok();
    Ok(ElementDef::Number {
        name,
        max_digits,
        align,
    })
}

fn parse_dynamic_text(input: ParseStream) -> Result<ElementDef> {
    let name: Ident = input.parse()?;
    let args;
    syn::parenthesized!(args in input);
    let w: LitInt = args.parse()?;
    args.parse::<Token![,]>()?;
    let h: LitInt = args.parse()?;
    let bake_settings = if args.peek(Token![,]) {
        args.parse::<Token![,]>()?;
        if !args.is_empty() {
            Some(args.parse::<Expr>()?)
        } else {
            None
        }
    } else {
        None
    };
    input.parse::<Token![,]>().ok();
    Ok(ElementDef::DynamicText {
        name,
        w,
        h,
        bake_settings,
    })
}

fn parse_dynamic_image(input: ParseStream) -> Result<ElementDef> {
    let name: Ident = input.parse()?;
    let args;
    syn::parenthesized!(args in input);
    let w: LitInt = args.parse()?;
    args.parse::<Token![,]>()?;
    let h: LitInt = args.parse()?;
    input.parse::<Token![,]>().ok();
    Ok(ElementDef::DynamicImage { name, w, h })
}

// -- Layout context ----------------------------------------------------------

struct LayoutCtx {
    counter: usize,
    consts: Vec<TokenStream>,
    first_draw_stmts: Vec<TokenStream>,
    dirty_draw_stmts: Vec<TokenStream>,
    static_texts: Vec<CollectedStatic>,
    dynamics: Vec<CollectedDynamic>,
    has_numbers: bool,
}

struct CollectedStatic {
    index: usize,
    text: LitStr,
}

struct CollectedDynamic {
    bit_index: usize,
    kind: DynamicKind,
}

#[allow(dead_code)]
enum DynamicKind {
    Number {
        name: Ident,
        max_digits: LitInt,
        align: Align,
    },
    Text {
        name: Ident,
    },
    Image {
        name: Ident,
    },
}

impl LayoutCtx {
    fn new() -> Self {
        Self {
            counter: 0,
            consts: Vec::new(),
            first_draw_stmts: Vec::new(),
            dirty_draw_stmts: Vec::new(),
            static_texts: Vec::new(),
            dynamics: Vec::new(),
            has_numbers: false,
        }
    }

    fn next_id(&mut self) -> usize {
        let id = self.counter;
        self.counter += 1;
        id
    }

    fn next_dynamic_bit(&self) -> usize {
        self.dynamics.len()
    }
}

// -- Element processing ------------------------------------------------------

fn dirty_const_name(name: &Ident) -> Ident {
    format_ident!("DIRTY_{}", name.to_string().to_uppercase())
}

/// Process a single element. Returns (width_expr, height_expr) token streams.
fn process_element(
    elem: &ElementDef,
    x_const: &Ident,
    y_const: &Ident,
    rect_idx: usize,
    ctx: &mut LayoutCtx,
    crate_path: &TokenStream,
    ui_tiles: &Expr,
    font: &Expr,
    bake_settings: &Expr,
) -> (TokenStream, TokenStream) {
    let rect_x = format_ident!("__RECT_{}_X", rect_idx);
    let rect_y = format_ident!("__RECT_{}_Y", rect_idx);
    let rect_w = format_ident!("__RECT_{}_W", rect_idx);
    let rect_h = format_ident!("__RECT_{}_H", rect_idx);

    match elem {
        ElementDef::StaticText { text } => {
            let st_idx = ctx.static_texts.len();
            let const_name = format_ident!("__STATIC_TEXT_{}", st_idx);
            ctx.static_texts.push(CollectedStatic {
                index: st_idx,
                text: text.clone(),
            });

            ctx.first_draw_stmts.push(quote! {
                {
                    let mut __rect = #crate_path::display::ui::UiRectangle::new(
                        #crate_path::fixnum::Rect::new(
                            #crate_path::fixnum::vec2(#rect_x, #rect_y),
                            #crate_path::fixnum::vec2(#rect_w, #rect_h),
                        ),
                        #ui_tiles,
                        __bg,
                    );
                    __rect.draw_tiles(
                        #crate_path::fixnum::vec2(#x_const, #y_const),
                        &#const_name,
                    );
                }
            });

            (
                quote! { #const_name.width as i32 },
                quote! { #const_name.height as i32 },
            )
        }

        ElementDef::StaticImage { expr } => {
            ctx.first_draw_stmts.push(quote! {
                {
                    let mut __rect = #crate_path::display::ui::UiRectangle::new(
                        #crate_path::fixnum::Rect::new(
                            #crate_path::fixnum::vec2(#rect_x, #rect_y),
                            #crate_path::fixnum::vec2(#rect_w, #rect_h),
                        ),
                        #ui_tiles,
                        __bg,
                    );
                    __rect.draw_tiles(
                        #crate_path::fixnum::vec2(#x_const, #y_const),
                        #expr,
                    );
                }
            });

            (
                quote! { {
                    const __TD: &#crate_path::display::tile_data::TileData = #expr;
                    __TD.width as i32
                } },
                quote! { {
                    const __TD: &#crate_path::display::tile_data::TileData = #expr;
                    __TD.height as i32
                } },
            )
        }

        ElementDef::Number {
            name,
            max_digits,
            align,
        } => {
            ctx.has_numbers = true;
            let bit = ctx.next_dynamic_bit();
            let dirty_name = dirty_const_name(name);

            let render_fn = if *align == Align::Right {
                quote! { #crate_path::display::ui::render_number_right }
            } else {
                quote! { #crate_path::display::ui::render_number_left }
            };

            ctx.dirty_draw_stmts.push(quote! {
                if __first || (__dirty & Self::#dirty_name != 0) {
                    let mut __rect = #crate_path::display::ui::UiRectangle::new(
                        #crate_path::fixnum::Rect::new(
                            #crate_path::fixnum::vec2(#rect_x, #rect_y),
                            #crate_path::fixnum::vec2(#rect_w, #rect_h),
                        ),
                        #ui_tiles,
                        __bg,
                    );
                    #render_fn(
                        &mut __rect,
                        #crate_path::fixnum::vec2(#x_const, #y_const),
                        #max_digits as usize,
                        self.#name,
                        &__DIGITS,
                        #ui_tiles,
                    );
                }
            });

            ctx.dynamics.push(CollectedDynamic {
                bit_index: bit,
                kind: DynamicKind::Number {
                    name: name.clone(),
                    max_digits: max_digits.clone(),
                    align: *align,
                },
            });

            (quote! { #max_digits as i32 }, quote! { 1i32 })
        }

        ElementDef::DynamicText {
            name,
            w,
            h,
            bake_settings: elem_settings,
        } => {
            let bit = ctx.next_dynamic_bit();
            let dirty_name = dirty_const_name(name);
            let settings_expr = if let Some(s) = elem_settings {
                quote! { #s }
            } else {
                quote! { #bake_settings }
            };

            ctx.dirty_draw_stmts.push(quote! {
                if __first || (__dirty & Self::#dirty_name != 0) {
                    let mut __rect = #crate_path::display::ui::UiRectangle::new(
                        #crate_path::fixnum::Rect::new(
                            #crate_path::fixnum::vec2(#rect_x, #rect_y),
                            #crate_path::fixnum::vec2(#rect_w, #rect_h),
                        ),
                        #ui_tiles,
                        __bg,
                    );
                    __rect.draw_text_dynamic(
                        #crate_path::fixnum::Rect::new(
                            #crate_path::fixnum::vec2(#x_const, #y_const),
                            #crate_path::fixnum::vec2(#w, #h),
                        ),
                        #font,
                        &self.#name,
                        #settings_expr,
                    );
                }
            });

            ctx.dynamics.push(CollectedDynamic {
                bit_index: bit,
                kind: DynamicKind::Text {
                    name: name.clone(),
                },
            });

            (quote! { #w as i32 }, quote! { #h as i32 })
        }

        ElementDef::DynamicImage { name, w, h } => {
            let bit = ctx.next_dynamic_bit();
            let dirty_name = dirty_const_name(name);

            ctx.dirty_draw_stmts.push(quote! {
                if __first || (__dirty & Self::#dirty_name != 0) {
                    let mut __rect = #crate_path::display::ui::UiRectangle::new(
                        #crate_path::fixnum::Rect::new(
                            #crate_path::fixnum::vec2(#rect_x, #rect_y),
                            #crate_path::fixnum::vec2(#rect_w, #rect_h),
                        ),
                        #ui_tiles,
                        __bg,
                    );
                    #crate_path::display::ui::clear_region(
                        &mut __rect,
                        #crate_path::fixnum::vec2(#x_const, #y_const),
                        #crate_path::fixnum::vec2(#w, #h),
                        #ui_tiles,
                    );
                    __rect.draw_tiles(
                        #crate_path::fixnum::vec2(#x_const, #y_const),
                        self.#name,
                    );
                }
            });

            ctx.dynamics.push(CollectedDynamic {
                bit_index: bit,
                kind: DynamicKind::Image {
                    name: name.clone(),
                },
            });

            (quote! { #w as i32 }, quote! { #h as i32 })
        }
    }
}

// -- Container processing ----------------------------------------------------

/// Process children in a row (direction=0) or column (direction=1).
/// Returns (width_const_ident, height_const_ident).
fn process_container(
    children: &[ContainerChild],
    direction: usize,
    base_x_expr: TokenStream,
    base_y_expr: TokenStream,
    rect_idx: usize,
    ctx: &mut LayoutCtx,
    crate_path: &TokenStream,
    ui_tiles: &Expr,
    font: &Expr,
    bake_settings: &Expr,
) -> (Ident, Ident) {
    let container_id = ctx.next_id();
    let base_x_const = format_ident!("__C_{}_BX", container_id);
    let base_y_const = format_ident!("__C_{}_BY", container_id);

    ctx.consts.push(quote! {
        const #base_x_const: i32 = #base_x_expr;
        const #base_y_const: i32 = #base_y_expr;
    });

    let mut cursor_parts: Vec<TokenStream> = Vec::new();
    let mut cross_sizes: Vec<TokenStream> = Vec::new();

    for child in children {
        let child_id = ctx.next_id();
        let child_x = format_ident!("__E_{}_X", child_id);
        let child_y = format_ident!("__E_{}_Y", child_id);

        let cursor_sum = if cursor_parts.is_empty() {
            quote! { 0i32 }
        } else {
            let parts = &cursor_parts;
            quote! { 0i32 #(+ #parts)* }
        };

        if direction == 0 {
            ctx.consts.push(quote! {
                const #child_x: i32 = #base_x_const + #cursor_sum;
                const #child_y: i32 = #base_y_const;
            });
        } else {
            ctx.consts.push(quote! {
                const #child_x: i32 = #base_x_const;
                const #child_y: i32 = #base_y_const + #cursor_sum;
            });
        }

        let (w_expr, h_expr) = match child {
            ContainerChild::Element(elem) => process_element(
                elem, &child_x, &child_y, rect_idx, ctx, crate_path, ui_tiles, font,
                bake_settings,
            ),
            ContainerChild::Row(row) => {
                let (w_id, h_id) = process_container(
                    &row.children, 0,
                    quote! { #child_x }, quote! { #child_y },
                    rect_idx, ctx, crate_path, ui_tiles, font, bake_settings,
                );
                (quote! { #w_id }, quote! { #h_id })
            }
            ContainerChild::Column(col) => {
                let (w_id, h_id) = process_container(
                    &col.children, 1,
                    quote! { #child_x }, quote! { #child_y },
                    rect_idx, ctx, crate_path, ui_tiles, font, bake_settings,
                );
                (quote! { #w_id }, quote! { #h_id })
            }
        };

        let child_w = format_ident!("__E_{}_W", child_id);
        let child_h = format_ident!("__E_{}_H", child_id);
        ctx.consts.push(quote! {
            const #child_w: i32 = #w_expr;
            const #child_h: i32 = #h_expr;
        });

        if direction == 0 {
            cursor_parts.push(quote! { #child_w });
            cross_sizes.push(quote! { #child_h });
        } else {
            cursor_parts.push(quote! { #child_h });
            cross_sizes.push(quote! { #child_w });
        }
    }

    let total_w = format_ident!("__C_{}_W", container_id);
    let total_h = format_ident!("__C_{}_H", container_id);

    if direction == 0 {
        let w_sum = if cursor_parts.is_empty() {
            quote! { 0i32 }
        } else {
            let parts = &cursor_parts;
            quote! { 0i32 #(+ #parts)* }
        };
        let h_max = const_max_expr(&cross_sizes);
        ctx.consts.push(quote! {
            const #total_w: i32 = #w_sum;
            const #total_h: i32 = #h_max;
        });
    } else {
        let w_max = const_max_expr(&cross_sizes);
        let h_sum = if cursor_parts.is_empty() {
            quote! { 0i32 }
        } else {
            let parts = &cursor_parts;
            quote! { 0i32 #(+ #parts)* }
        };
        ctx.consts.push(quote! {
            const #total_w: i32 = #w_max;
            const #total_h: i32 = #h_sum;
        });
    }

    (total_w, total_h)
}

fn const_max_expr(values: &[TokenStream]) -> TokenStream {
    if values.is_empty() {
        return quote! { 0i32 };
    }
    let mut result = values[values.len() - 1].clone();
    for i in (0..values.len() - 1).rev() {
        let a = &values[i];
        result = quote! { {
            const __A: i32 = #a;
            const __B: i32 = #result;
            if __A > __B { __A } else { __B }
        } };
    }
    result
}

// -- Top-level code generation -----------------------------------------------

pub fn generate(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let def: UiDef = match syn::parse(input) {
        Ok(d) => d,
        Err(e) => return e.to_compile_error().into(),
    };

    match generate_impl(def) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn generate_impl(def: UiDef) -> Result<TokenStream> {
    let crate_path = &def.crate_path;
    let struct_name = &def.name;
    let ui_tiles = &def.ui_tiles;
    let font = &def.font;
    let bake_settings = &def.bake_settings;

    let mut ctx = LayoutCtx::new();

    // Process each rect
    for (rect_idx, rect) in def.rects.iter().enumerate() {
        let rx = &rect.x;
        let ry = &rect.y;
        let rw = &rect.w;
        let rh = &rect.h;

        let rect_x = format_ident!("__RECT_{}_X", rect_idx);
        let rect_y = format_ident!("__RECT_{}_Y", rect_idx);
        let rect_w = format_ident!("__RECT_{}_W", rect_idx);
        let rect_h = format_ident!("__RECT_{}_H", rect_idx);

        ctx.consts.push(quote! {
            const #rect_x: i32 = #rx;
            const #rect_y: i32 = #ry;
            const #rect_w: i32 = #rw;
            const #rect_h: i32 = #rh;
        });

        // First draw: render 9-slice rect
        ctx.first_draw_stmts.push(quote! {
            {
                let mut __rect = #crate_path::display::ui::UiRectangle::new(
                    #crate_path::fixnum::Rect::new(
                        #crate_path::fixnum::vec2(#rect_x, #rect_y),
                        #crate_path::fixnum::vec2(#rect_w, #rect_h),
                    ),
                    #ui_tiles,
                    __bg,
                );
                __rect.draw();
            }
        });

        // Process children of this rect
        for child in &rect.children {
            match child {
                ContainerChild::Row(row) => {
                    let (bx, by) = row.coords.as_ref().map_or(
                        (quote! { 0i32 }, quote! { 0i32 }),
                        |(x, y)| (quote! { #x as i32 }, quote! { #y as i32 }),
                    );
                    process_container(
                        &row.children, 0, bx, by, rect_idx, &mut ctx, crate_path,
                        ui_tiles, font, bake_settings,
                    );
                }
                ContainerChild::Column(col) => {
                    let (bx, by) = col.coords.as_ref().map_or(
                        (quote! { 0i32 }, quote! { 0i32 }),
                        |(x, y)| (quote! { #x as i32 }, quote! { #y as i32 }),
                    );
                    process_container(
                        &col.children, 1, bx, by, rect_idx, &mut ctx, crate_path,
                        ui_tiles, font, bake_settings,
                    );
                }
                ContainerChild::Element(elem) => {
                    let elem_id = ctx.next_id();
                    let x_const = format_ident!("__E_{}_X", elem_id);
                    let y_const = format_ident!("__E_{}_Y", elem_id);
                    ctx.consts.push(quote! {
                        const #x_const: i32 = 0;
                        const #y_const: i32 = 0;
                    });
                    process_element(
                        elem, &x_const, &y_const, rect_idx, &mut ctx, crate_path,
                        ui_tiles, font, bake_settings,
                    );
                }
            }
        }
    }

    // -- Emit statics --------------------------------------------------------

    let static_text_decls: Vec<_> = ctx
        .static_texts
        .iter()
        .map(|st| {
            let const_name = format_ident!("__STATIC_TEXT_{}", st.index);
            let text = &st.text;
            quote! {
                static #const_name: #crate_path::display::tile_data::TileData =
                    #crate_path::ui_blit!(
                        #ui_tiles,
                        #crate_path::bake!(#font, #text, #bake_settings)
                    );
            }
        })
        .collect();

    let digits_decl = if ctx.has_numbers {
        quote! {
            static __DIGITS: [#crate_path::display::tile_data::TileData; 10] =
                #crate_path::bake_number_set!(#font, #bake_settings, #ui_tiles);
        }
    } else {
        quote! {}
    };

    // -- Emit struct fields, constructor, setters ----------------------------

    let mut struct_fields = Vec::new();
    let mut new_params = Vec::new();
    let mut new_field_inits = Vec::new();
    let mut setter_methods = Vec::new();
    let mut dirty_consts = Vec::new();

    for dyn_elem in &ctx.dynamics {
        let bit_val = 1u32 << dyn_elem.bit_index;

        match &dyn_elem.kind {
            DynamicKind::Number { name, .. } => {
                let dirty_name = dirty_const_name(name);
                let setter_name = format_ident!("set_{}", name);

                dirty_consts.push(quote! {
                    const #dirty_name: u32 = #bit_val;
                });
                struct_fields.push(quote! { #name: i32 });
                new_field_inits.push(quote! { #name: 0 });
                setter_methods.push(quote! {
                    pub fn #setter_name(&mut self, value: i32) {
                        if self.#name != value {
                            self.#name = value;
                            self.dirty |= Self::#dirty_name;
                        }
                    }
                });
            }
            DynamicKind::Text { name } => {
                let dirty_name = dirty_const_name(name);
                let setter_name = format_ident!("set_{}", name);

                dirty_consts.push(quote! {
                    const #dirty_name: u32 = #bit_val;
                });
                struct_fields.push(quote! { #name: alloc::string::String });
                new_field_inits.push(quote! { #name: alloc::string::String::new() });
                setter_methods.push(quote! {
                    pub fn #setter_name(&mut self, value: &str) {
                        if self.#name != value {
                            self.#name.clear();
                            self.#name.push_str(value);
                            self.dirty |= Self::#dirty_name;
                        }
                    }
                });
            }
            DynamicKind::Image { name } => {
                let dirty_name = dirty_const_name(name);
                let setter_name = format_ident!("set_{}", name);

                dirty_consts.push(quote! {
                    const #dirty_name: u32 = #bit_val;
                });
                struct_fields.push(quote! {
                    #name: &'static #crate_path::display::tile_data::TileData
                });
                new_params.push(quote! {
                    #name: &'static #crate_path::display::tile_data::TileData
                });
                new_field_inits.push(quote! { #name });
                setter_methods.push(quote! {
                    pub fn #setter_name(
                        &mut self,
                        value: &'static #crate_path::display::tile_data::TileData,
                    ) {
                        if !core::ptr::eq(self.#name, value) {
                            self.#name = value;
                            self.dirty |= Self::#dirty_name;
                        }
                    }
                });
            }
        }
    }

    // -- Assemble output -----------------------------------------------------

    let const_decls = &ctx.consts;
    let first_draw = &ctx.first_draw_stmts;
    let dirty_draw = &ctx.dirty_draw_stmts;

    let output = quote! {
        pub struct #struct_name {
            dirty: u32,
            #(#struct_fields,)*
        }

        #[allow(non_upper_case_globals, dead_code)]
        impl #struct_name {
            #(#dirty_consts)*

            pub fn new(#(#new_params,)*) -> Self {
                Self {
                    dirty: u32::MAX,
                    #(#new_field_inits,)*
                }
            }

            #(#setter_methods)*

            pub fn show(
                &mut self,
                __bg: &mut #crate_path::display::tiled::RegularBackground,
            ) {
                #(#static_text_decls)*
                #digits_decl

                #(#const_decls)*

                let __dirty = self.dirty;
                if __dirty == 0 {
                    return;
                }
                let __first = __dirty == u32::MAX;

                if __first {
                    #(#first_draw)*
                }

                #(#dirty_draw)*

                self.dirty = 0;
            }
        }
    };

    Ok(output)
}
