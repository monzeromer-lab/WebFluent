//! What shows data: avatars, skeletons, carousels, icon buttons, and pictures from the media pipeline.

use super::super::*;

impl JsCodegen {
    pub(in super::super) fn emit_avatar(&mut self, var: &str, ui: &UIElement, parent: &str) {
        // Through the scheme check like any other `src:`.
        let src = ui.args.iter().find_map(|a| match a {
            Arg::Named(k, v) if k == "src" => Some(v.clone()),
            _ => None,
        });
        let src = src.map(|v| self.picture_url_value(&v));
        let alt = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "alt" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });
        let initials = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "initials" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });

        let mut cls = "wf-avatar".to_string();
        for m in &ui.modifiers {
            match m.as_str() {
                "small" => cls.push_str(" wf-avatar--small"),
                "large" => cls.push_str(" wf-avatar--large"),
                "primary" => cls.push_str(" wf-avatar--primary"),
                _ => {}
            }
        }

        // What reads state follows it: the initials of a user who renames
        // themself, a picture that arrives. They were worked out once.
        let live = |v: String| {
            if self.is_reactive(&v) {
                format!("() => {v}")
            } else {
                v
            }
        };
        let src = src.map(live);
        let alt = alt.map(live);
        let initials = initials.map(live);
        // What any element takes is the circle's, beside the studio's id.
        let own = format!(
            "{}{}",
            self.root_attrs("Avatar", ui, &[]),
            self.wf_node_inline(ui)
        );
        if let Some(img_src) = src {
            let alt_val = alt.unwrap_or_else(|| "\"\"".to_string());
            self.emit_line(&format!(
                "const {var} = WF.el(\"div\", {{ className: \"{cls}\"{own} }}, WF.el(\"img\", {{ src: {img_src}, alt: {alt_val} }}));"
            ));
        } else if let Some(init) = initials {
            self.emit_line(&format!(
                "const {var} = WF.el(\"div\", {{ className: \"{cls}\"{own} }}, {init});"
            ));
        } else {
            self.emit_line(&format!(
                "const {var} = WF.el(\"div\", {{ className: \"{cls}\"{own} }});"
            ));
        }

        self.emit_root_handlers(var, ui, false);
        self.emit_line(&format!("{parent}.appendChild({var});"));
    }

    pub(in super::super) fn emit_skeleton(&mut self, var: &str, ui: &UIElement, parent: &str) {
        let height = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "height" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });
        let width = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "width" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });
        let size = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "size" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });

        let is_circle = ui.modifiers.iter().any(|m| m == "circle");
        let cls = if is_circle {
            "wf-skeleton wf-skeleton--circle"
        } else {
            "wf-skeleton"
        };

        let extra = self.root_attrs("Skeleton", ui, &[]);
        self.emit_line(&format!(
            "const {} = WF.el(\"div\", {{ className: \"{}\"{}{} }});",
            var,
            cls,
            extra,
            self.wf_node_inline(ui)
        ));
        // A size that reads state follows it.
        let set = |out: &mut Self, prop: &str, v: &str| {
            let line = format!("{var}.style.{prop} = {v};");
            if out.is_reactive(v) {
                out.emit_line(&format!("WF.effect(() => {{ {line} }});"));
            } else {
                out.emit_line(&line);
            }
        };
        if let Some(h) = &height {
            set(self, "height", h);
        }
        if let Some(w) = &width {
            set(self, "width", w);
        }
        if is_circle && let Some(s) = &size {
            if height.is_none() {
                set(self, "height", s);
            }
            if width.is_none() {
                set(self, "width", s);
            }
        }
        self.emit_root_handlers(var, ui, false);
        self.emit_line(&format!("{parent}.appendChild({var});"));
    }

    pub(in super::super) fn emit_carousel(&mut self, var: &str, ui: &UIElement, parent: &str) {
        // An author's `role:` and `aria-roledescription:` are kept by
        // `WF.carousel`, which writes its own only where none is given.
        let extra = self.root_attrs("Carousel", ui, &[]);
        self.emit_line(&format!(
            "const {} = WF.el(\"div\", {{ className: \"{}\"{}{} }});",
            var,
            self.class_attr("Carousel", ui),
            extra,
            self.wf_node_inline(ui)
        ));

        let track_var = self.fresh_var();
        self.emit_line(&format!(
            "const {track_var} = WF.el(\"div\", {{ className: \"wf-carousel__track\" }});"
        ));

        // The slides are the structure; everything else — the controls, the
        // rotation, the ARIA — is the runtime's `carousel`, so one place
        // holds the behaviour.
        for child in &ui.children {
            if let StatementKind::UIElement(ui_child) = &child.kind {
                if matches!(&ui_child.component, ComponentRef::SubComponent(p, s) if p == "Carousel" && s == "Slide")
                {
                    let slide_var = self.fresh_var();
                    // A slide's own `label:` names it to a screen reader in
                    // place of "n of N".
                    let label = ui_child.args.iter().find_map(|a| match a {
                        Arg::Named(k, v) if k == "label" => Some(self.emit_expr(v)),
                        _ => None,
                    });
                    let label_attr = label
                        .map(|l| format!(", \"aria-label\": {l}"))
                        .unwrap_or_default();
                    self.emit_line(&format!(
                        "const {} = WF.el(\"div\", {{ className: \"wf-carousel__slide\"{}{} }});",
                        slide_var,
                        label_attr,
                        self.wf_node_inline(ui_child)
                    ));
                    for c in &ui_child.children {
                        self.emit_statement_dom(c, &slide_var);
                    }
                    self.emit_line(&format!("{track_var}.appendChild({slide_var});"));
                } else {
                    self.emit_statement_dom(child, &track_var);
                }
            } else {
                self.emit_statement_dom(child, &track_var);
            }
        }

        self.emit_line(&format!("{var}.appendChild({track_var});"));

        let autoplay = ui.args.iter().any(|a| {
            matches!(a, Arg::Named(k, v) if k == "autoplay" && matches!(v, Expr::BoolLiteral(true)))
        });
        let interval = ui
            .args
            .iter()
            .find_map(|a| match a {
                Arg::Named(k, Expr::NumberLiteral(n)) if k == "interval" => {
                    Some(crate::project::builtin::carousel_interval(*n))
                }
                _ => None,
            })
            .unwrap_or(5000);
        let label = ui
            .args
            .iter()
            .find_map(|a| match a {
                Arg::Named(k, v) if k == "label" => Some(self.emit_expr(v)),
                _ => None,
            })
            .unwrap_or_else(|| "null".to_string());
        self.emit_line(&format!(
            "WF.carousel({var}, {{ autoplay: {autoplay}, interval: {interval}, label: {label} }});"
        ));

        self.emit_root_handlers(var, ui, false);
        self.emit_line(&format!("{parent}.appendChild({var});"));
    }

    pub(in super::super) fn emit_icon_button(&mut self, var: &str, ui: &UIElement, parent: &str) {
        let icon = ui
            .args
            .iter()
            .find_map(|a| {
                if let Arg::Named(k, v) = a {
                    if k == "icon" {
                        Some(self.emit_expr(v))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "\"\"".to_string());

        let label = ui.args.iter().find_map(|a| {
            if let Arg::Named(k, v) = a {
                if k == "label" {
                    Some(self.emit_expr(v))
                } else {
                    None
                }
            } else {
                None
            }
        });

        let mut cls = "wf-icon-btn".to_string();
        for m in &ui.modifiers {
            match m.as_str() {
                "small" => cls.push_str(" wf-icon-btn--small"),
                "large" => cls.push_str(" wf-icon-btn--large"),
                "primary" => cls.push_str(" wf-icon-btn--primary"),
                "danger" => cls.push_str(" wf-icon-btn--danger"),
                _ => {}
            }
        }
        // A `class:` joins the engine's classes; it used to replace them, so
        // the button lost its size and shape. One that reads a value is
        // followed by the generic path, never written as an attribute.
        for extra in crate::project::builtin::author_classes(&ui.args) {
            cls.push(' ');
            cls.push_str(&extra);
        }

        let icon_attr = if self.is_reactive(&icon) {
            format!("() => {icon}")
        } else {
            icon.clone()
        };
        let mut btn_attrs = format!("className: \"{cls}\", \"data-icon\": {icon_attr}");
        if let Some(l) = &label {
            let l = if self.is_reactive(l) {
                format!("() => {l}")
            } else {
                l.clone()
            };
            btn_attrs.push_str(&format!(", \"aria-label\": {l}"));
        }
        // Its label is its tooltip too, unless its author writes one: two
        // `title` keys kept the last on the live page and the first in the
        // painted one.
        if !Self::gives(ui, "title") {
            let title = label.as_deref().unwrap_or(&icon);
            let title = if self.is_reactive(title) {
                format!("() => {title}")
            } else {
                title.to_string()
            };
            btn_attrs.push_str(&format!(", title: {title}"));
        }

        // Every other named argument is an attribute, as on a Button: `type`,
        // `disabled`, `aria-haspopup`, `data-variant`. A value that reads
        // state is a thunk the runtime keeps in step with it. What every
        // element takes beside them — a lifetime, a drag, motion — is the
        // shared tail's: written here too, a `mount:` was run once more as
        // an attribute's value, with no element.
        btn_attrs.push_str(&self.root_attrs("IconButton", ui, &[]));

        for handler in &ui.events {
            let body = self.emit_event_body(handler);
            btn_attrs.push_str(&format!(
                ", \"on:{}\": {} => {{ {} }}",
                handler.event,
                self.handler_head(handler, "event"),
                body
            ));
        }
        if let Some(entry) = self.wf_node_entry(ui) {
            btn_attrs.push_str(&format!(", {entry}"));
        }

        self.emit_line(&format!(
            "const {var} = WF.el(\"button\", {{ {btn_attrs} }}, WF.el(\"span\", {{ className: \"wf-icon\", \"data-icon\": {icon} }}));"
        ));
        self.emit_line(&format!("{parent}.appendChild({var});"));
    }

    /// A `Video` with captions, or an `Audio` with a transcript: what a
    /// reader who cannot hear it needs, beside it rather than instead.
    pub(in super::super) fn emit_media(
        &mut self,
        name: &str,
        var: &str,
        attrs: &str,
        ui: &UIElement,
        parent: &str,
    ) {
        let tag = crate::project::builtin::builtin_to_html(name).0;
        self.emit_line(&format!("const {var} = WF.el(\"{tag}\", {attrs});"));
        // `on ended`, `on timeupdate`: the player's, as on one without
        // captions.
        self.emit_root_handlers(var, ui, false);
        let named = |key: &str| {
            ui.args.iter().find_map(|a| match a {
                Arg::Named(k, v) if k == key => Some(v),
                _ => None,
            })
        };
        if let Some(captions) = named("captions") {
            let src = self.emit_expr(captions);
            self.emit_line(&format!(
                "{var}.appendChild(WF.el(\"track\", {{ kind: \"captions\", src: {src}, srclang: document.documentElement.lang || \"en\", default: true }}));"
            ));
        }
        self.emit_line(&format!("{parent}.appendChild({var});"));
        if let Some(transcript) = named("transcript") {
            let href = self.emit_expr(transcript);
            self.emit_line(&format!(
                "{parent}.appendChild(WF.el(\"a\", {{ className: \"wf-transcript\", href: {href} }}, \"Read the transcript\"));"
            ));
        }
    }

    /// `Image(hero, alt: "…", sizes: "…", placeholder: .blur)` — the
    /// picture the build made, at every width it made.
    pub(in super::super) fn emit_picture(&mut self, var: &str, ui: &UIElement, parent: &str) {
        let named = |key: &str| {
            ui.args.iter().find_map(|a| match a {
                Arg::Named(k, v) if k == key => Some(v),
                _ => None,
            })
        };
        let positional = ui.args.iter().find_map(|a| match a {
            Arg::Positional(v) => Some(v),
            _ => None,
        });
        let source = positional
            .or_else(|| named("source"))
            .map(|v| self.emit_expr(v))
            .unwrap_or_default();
        let mut opts: Vec<String> = Vec::new();
        for key in ["alt", "sizes", "loading", "className"] {
            if let Some(value) = named(key) {
                opts.push(format!("{key}: {}", self.emit_expr(value)));
            }
        }
        // `placeholder: .blur` is a case by the time it is here.
        let placeholder = named("placeholder")
            .and_then(|v| match v {
                Expr::EnumCase(case) => Some(case.clone()),
                _ => None,
            })
            .or_else(|| {
                ui.modifiers
                    .iter()
                    .find(|m| matches!(m.as_str(), "blur" | "color" | "none"))
                    .cloned()
            });
        if let Some(kind) = placeholder {
            opts.push(format!("placeholder: \"{kind}\""));
        }
        let classes = crate::project::builtin::builtin_to_html("Image").1;
        opts.push(format!("className: \"{classes}\""));
        // What any element takes — an `id:`, `aria-*`, `data-*`, a `ref:`
        // — and its handlers are the image's, inside the `<picture>`, where
        // its `alt:` is and its `load` fires. The build's own width, height
        // and sources stay the picture's.
        let mut own = self.root_attrs(
            "Image",
            ui,
            &[
                "alt",
                "sizes",
                "loading",
                "className",
                "source",
                "placeholder",
                "src",
                "width",
                "height",
            ],
        );
        let mut gestures = Vec::new();
        for handler in &ui.events {
            if GESTURE_EVENTS.contains(&handler.event.as_str()) {
                gestures.push(handler);
                continue;
            }
            let body = self.emit_event_body(handler);
            own.push_str(&format!(
                ", \"on:{}\": {} => {{ {body} }}",
                handler.event,
                self.handler_head(handler, "event")
            ));
        }
        if let Some(own) = own.strip_prefix(", ") {
            opts.push(format!("attrs: {{ {own} }}"));
        }
        self.emit_line(&format!(
            "const {var} = WF.picture({source}, {{ {} }});",
            opts.join(", ")
        ));
        // A gesture is the picture's, as a drag on it is.
        for handler in gestures {
            let body = self.emit_event_body(handler);
            let line = self.element_listener(var, handler, &body);
            self.emit_line(&line);
        }
        self.emit_line(&format!("{parent}.appendChild({var});"));
    }
}
