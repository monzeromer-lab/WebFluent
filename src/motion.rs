//! Motion as data: the animations an element can play, how fast and how
//! paced, the route transition, the project's default — and how each is
//! written on an element, so an inspector offers what the engine has and
//! writes what the language reads.
//!
//! The animations are the registry's (`animate:` and `exit:` take the same
//! cases), each with the keyframes and the rule the engine's own stylesheet
//! plays it with, so a tile can play exactly what the page will; the
//! project's own `animation` declarations come after. The eases are the
//! registry's, with the timing function each stands for and its value in
//! the baseline tokens; the durations are the speed flags' and the duration
//! tokens'.
//!
//! A reader who asks for less motion gets none, and an editor previews
//! that by setting `data-wf-motion="reduce"` on the page's root — which the
//! stylesheet and the runtime honour as they honour the reader's setting.

use serde::Serialize;

use crate::parser::ast::{AnimationDecl, Declaration, Program};

/// One animation.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Animation {
    /// As `animate:` takes it: `fadeIn`.
    pub name: String,
    pub summary: String,
    /// The `@keyframes` rule that plays it, as the page's stylesheet has it.
    pub keyframes: String,
    /// The class rule that applies it: its length, its pacing, whether it
    /// repeats.
    pub rule: String,
    /// Whether it is one an element leaves with (`fadeOut`, `scaleOut`).
    pub leaves: bool,
    /// Whether it repeats until stopped (`pulse`, `spin`).
    pub repeats: bool,
    /// One the project declares (`animation Pulse { … }`).
    pub project: bool,
}

/// One named ease.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Ease {
    /// As `easing:` takes it: `.spring`.
    pub name: String,
    pub summary: String,
    /// The timing function it stands for: `var(--ease-spring)`, `ease-in`.
    pub css: String,
    /// What that is in the baseline, when it is a token.
    pub value: Option<String>,
}

/// One length an animation runs for.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Speed {
    /// `fast`, `normal`, `slow`.
    pub name: String,
    /// The token it is: `animation-duration-fast`.
    pub token: String,
    /// Its baseline value: `150ms`.
    pub value: String,
    /// The flag that writes it, where there is one: `.fast`.
    pub flag: Option<String>,
}

/// What an element does when it says nothing: the config's `motion`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Default {
    pub duration: String,
    pub easing: String,
    /// Whether the config sets them, or they are the engine's.
    pub from_config: bool,
}

/// Everything a Motion section offers.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Motion {
    pub animations: Vec<Animation>,
    pub eases: Vec<Ease>,
    pub speeds: Vec<Speed>,
    pub default: Default,
    /// `Router(transition: …)`'s cases.
    pub route_transitions: Vec<String>,
    /// The attribute an editor sets on the page's root to show it as a
    /// reader who asked for less motion sees it.
    pub reduced_attribute: &'static str,
}

/// The attribute, and the value, that previews reduced motion.
pub const REDUCED: &str = "data-wf-motion=\"reduce\"";

/// The `@keyframes wf-<name>` rule in `css`, braces and all.
fn keyframes_in(css: &str, name: &str) -> Option<String> {
    let start = css.find(&format!("@keyframes {name} {{"))?;
    let mut depth = 0;
    for (i, c) in css[start..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(css[start..start + i + 1].to_string());
                }
            }
            _ => {}
        }
    }
    None
}

/// The `.wf-animate-<name> { … }` rule in `css`.
fn rule_in(css: &str, name: &str) -> Option<String> {
    let start = css.find(&format!(".wf-animate-{name} {{"))?;
    let end = css[start..].find('}')?;
    Some(css[start..start + end + 1].to_string())
}

fn cases(prop: &str) -> &'static [crate::registry::CaseSig] {
    crate::registry::UNIVERSAL_PROPS
        .iter()
        .find(|p| p.name == prop)
        .and_then(|p| match p.ty {
            crate::registry::PropType::Enum(c) => Some(c),
            _ => None,
        })
        .unwrap_or(&[])
}

/// The project's own animation, as the build's stylesheet writes it.
fn project_animation(a: &AnimationDecl) -> Animation {
    let (keyframes, rule) = crate::codegen::scoped_css::animation_css(a);
    Animation {
        name: a.name.clone(),
        summary: a.doc.clone().unwrap_or_default(),
        keyframes,
        rule,
        leaves: false,
        repeats: false,
        project: true,
    }
}

/// What a Motion section offers in `program`, with `config` for the
/// project's default.
pub fn catalogue(program: Option<&Program>, config: &crate::config::MotionConfig) -> Motion {
    let css = crate::themes::components::component_css();
    let tokens = crate::themes::tokens::default_tokens();
    let mut animations: Vec<Animation> = cases("animate")
        .iter()
        .map(|c| {
            let frames = keyframes_in(css, &format!("wf-{}", c.name)).unwrap_or_default();
            let rule = rule_in(css, c.name).unwrap_or_default();
            Animation {
                name: c.name.to_string(),
                summary: c.summary.to_string(),
                repeats: rule.contains("infinite"),
                leaves: c.name.ends_with("Out") || c.name == "collapse",
                keyframes: frames,
                rule,
                project: false,
            }
        })
        .collect();
    if let Some(p) = program {
        for d in &p.declarations {
            if let Declaration::Animation(a) = d {
                animations.push(project_animation(a));
            }
        }
    }
    let eases = cases("easing")
        .iter()
        .map(|c| {
            let css = crate::codegen::builtin::easing_css(c.name).to_string();
            let value = css
                .strip_prefix("var(--")
                .and_then(|n| n.strip_suffix(')'))
                .and_then(|n| tokens.get(n).cloned());
            Ease {
                name: c.name.to_string(),
                summary: c.summary.to_string(),
                css,
                value,
            }
        })
        .collect();
    let speeds = ["fast", "normal", "slow"]
        .iter()
        .map(|s| {
            let token = format!("animation-duration-{s}");
            Speed {
                name: s.to_string(),
                value: tokens.get(&token).cloned().unwrap_or_default(),
                token,
                flag: (*s != "normal").then(|| format!(".{s}")),
            }
        })
        .collect();
    let route_transitions = crate::registry::component("Router")
        .and_then(|r| r.props.iter().find(|p| p.name == "transition"))
        .map(|p| match p.ty {
            crate::registry::PropType::Enum(c) => c.iter().map(|c| c.name.to_string()).collect(),
            _ => Vec::new(),
        })
        .unwrap_or_default();
    Motion {
        animations,
        eases,
        speeds,
        default: Default {
            duration: config.duration.clone().unwrap_or_else(|| {
                tokens
                    .get("animation-duration-normal")
                    .cloned()
                    .unwrap_or_default()
            }),
            easing: config.easing.clone().unwrap_or_else(|| {
                tokens
                    .get("animation-easing-default")
                    .cloned()
                    .unwrap_or_default()
            }),
            from_config: config.duration.is_some() || config.easing.is_some(),
        },
        route_transitions,
        reduced_attribute: REDUCED,
    }
}

/// How an animation is written on an element whose enum props take these
/// cases: the flag `.fadeIn`, unless one of them takes the word too, when
/// a bare `.fadeIn` would not say which — then `animate: .fadeIn` (or
/// `exit:` for one it leaves with). The project's own animation has no
/// flag: it is always `animate: .Drop`.
pub fn spelling(prop: &str, animation: &str, other_cases: &[&str]) -> String {
    let built_in = crate::themes::prune::ANIMATIONS.contains(&animation);
    if prop == "animate" && built_in && !other_cases.contains(&animation) {
        format!(".{animation}")
    } else {
        format!("{prop}: .{animation}")
    }
}

/// How a duration is written: `.fast` or `.slow` where it is that speed,
/// otherwise `duration: "240ms"`.
pub fn duration_spelling(ms: u32) -> String {
    let tokens = crate::themes::tokens::default_tokens();
    for s in ["fast", "slow"] {
        if tokens
            .get(&format!("animation-duration-{s}"))
            .map(String::as_str)
            == Some(&format!("{ms}ms"))
        {
            return format!(".{s}");
        }
    }
    format!("duration: \"{ms}ms\"")
}

/// Why `stagger:` means nothing on this node, or nothing when it does: it
/// is the delay between the items of a list, so only a node inside a `for`
/// has something to stagger.
pub fn stagger_unavailable(node: &crate::codegen::node_id::NodeInfo) -> Option<&'static str> {
    (!node.template()).then_some("Only an element inside a `for` has items to stagger.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_animations_are_the_registry_s_with_what_plays_them() {
        let m = catalogue(None, &Default_config());
        let names: Vec<&str> = m.animations.iter().map(|a| a.name.as_str()).collect();
        for n in [
            "fadeIn",
            "fadeOut",
            "slideUp",
            "slideDown",
            "slideLeft",
            "slideRight",
            "scaleIn",
            "scaleOut",
            "bounce",
            "shake",
            "pulse",
            "spin",
        ] {
            assert!(names.contains(&n), "{n}");
        }
        for a in &m.animations {
            assert!(
                !a.rule.is_empty(),
                "{} has a class rule in the engine's sheet",
                a.name
            );
            assert!(a.rule.contains("animation:"), "{}", a.rule);
            if a.name != "spin" {
                assert!(
                    a.keyframes
                        .starts_with(&format!("@keyframes wf-{} {{", a.name)),
                    "{}",
                    a.name
                );
            }
        }
        let pulse = m.animations.iter().find(|a| a.name == "pulse").unwrap();
        assert!(pulse.repeats && !pulse.leaves);
        assert!(
            m.animations
                .iter()
                .find(|a| a.name == "fadeOut")
                .unwrap()
                .leaves
        );
        let spring = m.eases.iter().find(|e| e.name == "spring").unwrap();
        assert_eq!(spring.css, "var(--ease-spring)");
        assert!(
            spring
                .value
                .as_deref()
                .is_some_and(|v| v.starts_with("linear("))
        );
        assert_eq!(
            m.speeds
                .iter()
                .map(|s| (s.name.as_str(), s.value.as_str()))
                .collect::<Vec<_>>(),
            [("fast", "150ms"), ("normal", "300ms"), ("slow", "500ms")]
        );
        assert_eq!(m.route_transitions, ["none", "fade", "slide"]);
        assert_eq!(
            (m.default.duration.as_str(), m.default.from_config),
            ("300ms", false)
        );
    }

    #[allow(non_snake_case)]
    fn Default_config() -> crate::config::MotionConfig {
        crate::config::MotionConfig::default()
    }

    #[test]
    fn each_is_written_as_the_language_reads_it() {
        assert_eq!(spelling("animate", "fadeIn", &[]), ".fadeIn");
        assert_eq!(
            spelling("animate", "fadeIn", &["fadeIn"]),
            "animate: .fadeIn"
        );
        assert_eq!(spelling("exit", "fadeOut", &[]), "exit: .fadeOut");
        assert_eq!(spelling("animate", "Drop", &[]), "animate: .Drop");
        assert_eq!(duration_spelling(150), ".fast");
        assert_eq!(duration_spelling(500), ".slow");
        assert_eq!(duration_spelling(240), "duration: \"240ms\"");
        let config: crate::config::MotionConfig =
            serde_json::from_str(r#"{ "duration": "180ms" }"#).unwrap();
        let m = catalogue(None, &config);
        assert_eq!(
            (m.default.duration.as_str(), m.default.from_config),
            ("180ms", true)
        );
        let p = crate::syntax::parse_source("animation Wobble {\n    from {\n        rotate: 0deg\n    }\n    to {\n        rotate: 8deg\n    }\n}\n", "t.wf").unwrap();
        let m = catalogue(Some(&p), &Default_config());
        let w = m.animations.last().unwrap();
        assert_eq!((w.name.as_str(), w.project), ("Wobble", true));
        assert_eq!(
            w.keyframes,
            "@keyframes Wobble { from { rotate: 0deg; } to { rotate: 8deg; } }"
        );
    }
}
