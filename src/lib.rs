use std::{fmt::Display, io::Read};

use svg::node::element::path::{Command, Data, Position::Absolute};

/// Represents a single tikz `\draw` command
#[derive(Default)]
pub struct TikzDraw {
    attributes: Vec<Attribute>,
    path_sections: Vec<PathSection>,
}

pub enum Attribute {
    Setting(String),
    Param(String, String),
}

impl Attribute {
    pub fn setting<S: Into<String>>(s: S) -> Self {
        Attribute::Setting(s.into())
    }

    pub fn param<K: Into<String>, V: Into<String>>(k: K, v: V) -> Self {
        Attribute::Param(k.into(), v.into())
    }
}

impl Display for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Attribute::Setting(s) => write!(f, "{}", s),
            Attribute::Param(k, v) => write!(f, "{}={}", k, v),
        }
    }
}

pub fn attributes_to_tikz(attrs: &[Attribute]) -> String {
    attrs
        .iter()
        .map(Attribute::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

impl Display for TikzDraw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let attrs = attributes_to_tikz(&self.attributes);
        let path = self
            .path_sections
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        write!(f, "\\draw[{}] {} ;", attrs, path)
    }
}

pub struct Point(f32, f32);

impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:.4}, {:.4})", self.0, self.1)
    }
}

pub enum PathSection {
    Move(Point),
    Line(Point),
    Curve(Point, Point, Point),
    Cycle,
}

impl Display for PathSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathSection::Move(p) => write!(f, "{}", p),
            PathSection::Line(p) => write!(f, "--{}", p),
            PathSection::Curve(c1, c2, p) => write!(f, ".. controls {} and {} .. {}", c1, c2, p),
            PathSection::Cycle => write!(f, "--cycle"),
        }
    }
}

impl PathSection {
    pub fn from_svg(cmd: &Command) -> Self {
        match cmd {
            Command::Move(Absolute, params) => PathSection::Move(Point(params[0], params[1])),
            Command::Line(Absolute, params) => PathSection::Line(Point(params[0], params[1])),
            Command::CubicCurve(Absolute, params) => PathSection::Curve(
                Point(params[0], params[1]),
                Point(params[2], params[3]),
                Point(params[4], params[5]),
            ),
            Command::Close => PathSection::Cycle,
            _command => panic!("not yet supported: {:?}", cmd),
        }
    }
}

pub fn parse_svg<R: Read>(input: R) -> anyhow::Result<TikzDraw> {
    let mut result = TikzDraw::default();
    let input = std::io::read_to_string(input)?;
    // for now, just add the same attributes every time
    result.attributes.push(Attribute::setting("fill"));
    result.attributes.push(Attribute::setting("even odd rule"));
    result.attributes.push(Attribute::param("line width", "1"));

    for event in svg::read(&input)? {
        use svg::node::element::tag;
        #[allow(clippy::single_match)]
        match event {
            svg::parser::Event::Tag(tag::Path, _, attrs) => {
                let data = attrs.get("d").unwrap();
                let data = Data::parse(data).unwrap();
                result.path_sections = data.iter().map(PathSection::from_svg).collect();
                break;
            }
            _ => {} // ignore everything esle
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use super::*;

    #[test]
    fn test_lambda_icon() -> anyhow::Result<()> {
        let f = File::open("testfiles/lambda.svg")?;
        let tikz = parse_svg(f)?;
        println!("{}", tikz);

        Ok(())
    }
}
