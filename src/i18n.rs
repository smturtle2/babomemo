use std::{collections::BTreeMap, str::FromStr};

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use fluent_langneg::{NegotiationStrategy, negotiate_languages};
use rust_embed::RustEmbed;
use unic_langid::LanguageIdentifier;

#[derive(RustEmbed)]
#[folder = "locales/"]
struct Locales;

pub struct I18n {
    primary: FluentBundle<FluentResource>,
    fallback: Option<FluentBundle<FluentResource>>,
}

impl I18n {
    pub fn load() -> Result<Self, String> {
        let default_source =
            Locales::get("default").ok_or_else(|| "missing default locale resource".to_owned())?;
        let default_tag = std::str::from_utf8(default_source.data.as_ref())
            .map_err(|error| error.to_string())?
            .trim()
            .parse::<LanguageIdentifier>()
            .map_err(|error| error.to_string())?;

        let mut resources = BTreeMap::new();
        for path in Locales::iter() {
            let Some(tag) = path.strip_suffix(".ftl") else {
                continue;
            };
            if let Ok(language) = LanguageIdentifier::from_str(tag) {
                resources.insert(language, path.into_owned());
            }
        }
        if !resources.contains_key(&default_tag) {
            return Err("default locale has no Fluent resource".to_owned());
        }

        let requested = sys_locale::get_locales()
            .filter_map(|locale| normalize_locale(&locale).parse::<LanguageIdentifier>().ok())
            .collect::<Vec<_>>();
        let available = resources.keys().cloned().collect::<Vec<_>>();
        let negotiated = negotiate_languages(
            &requested,
            &available,
            Some(&default_tag),
            NegotiationStrategy::Filtering,
        );
        let selected = negotiated.first().copied().unwrap_or(&default_tag).clone();

        let primary_path = resources
            .get(&selected)
            .ok_or_else(|| "negotiated locale has no Fluent resource".to_owned())?;
        let primary = make_bundle(&selected, primary_path)?;
        let fallback = (selected != default_tag)
            .then(|| {
                let path = resources
                    .get(&default_tag)
                    .ok_or_else(|| "default locale has no Fluent resource".to_owned())?;
                make_bundle(&default_tag, path)
            })
            .transpose()?;
        Ok(Self { primary, fallback })
    }

    pub fn text(&self, id: &str) -> String {
        self.format(id, None)
    }

    pub fn format(&self, id: &str, args: Option<&FluentArgs<'_>>) -> String {
        format_message(&self.primary, id, args)
            .or_else(|| {
                self.fallback
                    .as_ref()
                    .and_then(|bundle| format_message(bundle, id, args))
            })
            .unwrap_or_else(|| id.to_owned())
    }
}

fn normalize_locale(locale: &str) -> String {
    locale
        .split(['.', '@'])
        .next()
        .unwrap_or(locale)
        .replace('_', "-")
}

fn make_bundle(
    language: &LanguageIdentifier,
    path: &str,
) -> Result<FluentBundle<FluentResource>, String> {
    let source = Locales::get(path).ok_or_else(|| format!("missing locale resource: {path}"))?;
    let source = String::from_utf8(source.data.into_owned()).map_err(|error| error.to_string())?;
    let resource = FluentResource::try_new(source)
        .map_err(|(_, errors)| format!("invalid locale resource {path}: {errors:?}"))?;
    let mut bundle = FluentBundle::new(vec![language.clone()]);
    bundle.set_use_isolating(false);
    bundle
        .add_resource(resource)
        .map_err(|errors| format!("invalid locale messages {path}: {errors:?}"))?;
    Ok(bundle)
}

fn format_message(
    bundle: &FluentBundle<FluentResource>,
    id: &str,
    args: Option<&FluentArgs<'_>>,
) -> Option<String> {
    let message = bundle.get_message(id)?;
    let pattern = message.value()?;
    let mut errors = Vec::new();
    Some(
        bundle
            .format_pattern(pattern, args, &mut errors)
            .into_owned(),
    )
}
