//! A small reader for the PDFs this crate writes.
//!
//! Every assertion in this directory is made against the *emitted document*,
//! parsed back, rather than against the values that went into the emitter.
//! That distinction is the whole point of these tests.
//!
//! The obvious way to enforce "export draws the kernel's cut line and does
//! not compute its own" is to compare the emitter's input to the cut line —
//! but that compares a value to itself and passes no matter what the emitter
//! then does with it. Every real breach is downstream: a re-flatten inside
//! the writer, a stray scale factor, coordinates rounded on the way out,
//! points dropped at a tile edge. None of those are visible from the input
//! side. Reading the content stream back is the only place they show up.
//!
//! This parser understands exactly the subset this crate emits. It is not a
//! PDF implementation and should never grow into one.

// Each integration-test binary compiles its own copy of this module and uses
// a different part of it.
#![allow(dead_code)]

use patal_export::{Mm, Pt, MM_PER_PT};

/// One page, as read back out of the file.
pub struct ParsedPage {
    /// The `% patal-...` comment lines, verbatim.
    pub comments: Vec<String>,
    /// Every stroked polyline on the page, in page points, in draw order.
    pub polylines: Vec<Vec<(Pt, Pt)>>,
    /// Every stroked `re` rectangle: x, y, width, height in points.
    pub rects: Vec<(f64, f64, f64, f64)>,
    /// Every clipping rectangle (`re W n`). Kept apart from the stroked ones
    /// because the clip is the structural guarantee that pattern geometry
    /// cannot reach the calibration strip, and a test of that guarantee must
    /// not accidentally match a decorative frame.
    pub clips: Vec<(f64, f64, f64, f64)>,
    /// Every literal string drawn with `Tj`.
    pub texts: Vec<String>,
    /// The same strings with the page-space point they were drawn at.
    ///
    /// Kept separately because only one test needs positions, and only for
    /// one purpose: a registration cross's label is drawn at a fixed offset
    /// from the cross itself, so the label's own coordinates are how a test
    /// learns which model point a given `x1y0` claims to mark — without
    /// which "matching labels coincide" cannot be checked against the
    /// document, only against the code that wrote it.
    pub placed_texts: Vec<(Pt, Pt, String)>,
    /// The page's `/MediaBox`, in points.
    pub media_box: (f64, f64),
}

impl ParsedPage {
    fn tile_comment(&self) -> Option<&String> {
        self.comments.iter().find(|c| c.starts_with("patal-tile "))
    }

    /// The value of a whitespace-free `key=value` field in the `patal-tile`
    /// comment.
    pub fn tile_field(&self, key: &str) -> Option<String> {
        let needle = format!("{key}=");
        self.tile_comment()?
            .split_whitespace()
            .find_map(|token| token.strip_prefix(&needle))
            .map(str::to_string)
    }

    /// The piece name, which is quoted because it may contain spaces.
    pub fn tile_piece(&self) -> Option<String> {
        let comment = self.tile_comment()?;
        let rest = comment.split_once("piece=\"")?.1;
        Some(rest.split_once('"')?.0.to_string())
    }

    /// The model coordinate this sheet's drawable window starts at, as the
    /// document itself states it.
    pub fn tile_origin_mm(&self) -> Option<(Mm, Mm)> {
        let raw = self.tile_field("origin_mm")?;
        let (x, y) = raw.split_once(',')?;
        Some((Mm(x.parse().ok()?), Mm(y.parse().ok()?)))
    }

    pub fn window_origin_mm(&self) -> Option<(Mm, Mm)> {
        let raw = self.tile_field("window_origin_mm")?;
        let (x, y) = raw.split_once(',')?;
        Some((Mm(x.parse().ok()?), Mm(y.parse().ok()?)))
    }

    /// Converts a point on this sheet back into model millimetres, using only
    /// what the sheet itself declares about where its window sits.
    ///
    /// This is the arithmetic a person performs by laying one sheet on top of
    /// another, and doing it from the document's own comments rather than
    /// from the layout code is what makes an assertion about assembly worth
    /// making.
    pub fn to_model_mm(&self, x: Pt, y: Pt) -> Option<(Mm, Mm)> {
        let (origin_x, origin_y) = self.tile_origin_mm()?;
        let (window_x, window_y) = self.window_origin_mm()?;
        Some((
            Mm(x.get() * MM_PER_PT - window_x.get() + origin_x.get()),
            Mm(y.get() * MM_PER_PT - window_y.get() + origin_y.get()),
        ))
    }

    /// The longest polyline on the page.
    ///
    /// On a tile page that is the cut line: the registration crosses are
    /// two-point segments and the sewing line has the same vertex count as
    /// the cut line but is drawn first, so "longest, last on a tie" picks the
    /// cut line without the test needing to know the draw order.
    pub fn longest_polyline(&self) -> Option<&Vec<(Pt, Pt)>> {
        let max = self.polylines.iter().map(Vec::len).max()?;
        self.polylines.iter().rfind(|p| p.len() == max)
    }
}

/// Splits a rendered PDF into pages and reads each one's content stream.
pub fn parse(pdf: &[u8]) -> Vec<ParsedPage> {
    let text = latin1(pdf);
    let media_boxes: Vec<(f64, f64)> = text
        .match_indices("/MediaBox [0 0 ")
        .map(|(index, marker)| {
            let rest = &text[index + marker.len()..];
            let end = rest.find(']').expect("a MediaBox is closed");
            let mut parts = rest[..end].split_whitespace();
            (
                parts.next().unwrap().parse().unwrap(),
                parts.next().unwrap().parse().unwrap(),
            )
        })
        .collect();

    let mut pages = Vec::new();
    // Split on "\nstream\n", not "stream\n": the latter also matches inside
    // "endstream\n" and yields twice as many chunks as there are pages.
    for (index, chunk) in text.split("\nstream\n").skip(1).enumerate() {
        let body = chunk
            .split("\nendstream")
            .next()
            .expect("a stream is closed");
        pages.push(parse_content(
            body,
            *media_boxes.get(index).expect("one MediaBox per page"),
        ));
    }
    pages
}

/// The content streams this crate writes are pure ASCII by construction (see
/// `pdf::pdf_string`), so a byte-for-char decode is exact and, unlike
/// `from_utf8_lossy`, does not change any offsets.
fn latin1(bytes: &[u8]) -> String {
    bytes.iter().map(|b| *b as char).collect()
}

fn parse_content(body: &str, media_box: (f64, f64)) -> ParsedPage {
    let mut page = ParsedPage {
        comments: Vec::new(),
        polylines: Vec::new(),
        rects: Vec::new(),
        clips: Vec::new(),
        texts: Vec::new(),
        placed_texts: Vec::new(),
        media_box,
    };
    let mut current: Vec<(Pt, Pt)> = Vec::new();

    for line in body.lines() {
        let line = line.trim();
        if let Some(comment) = line.strip_prefix("% ") {
            page.comments.push(comment.to_string());
            continue;
        }
        if line.starts_with("BT ") {
            // `BT /F1 <size> Tf <x> <y> Td (label) Tj ET`
            if let Some(open) = line.find('(') {
                if let Some(close) = line.rfind(") Tj") {
                    let label = line[open + 1..close].to_string();
                    // Located by finding `Td` rather than by a fixed index,
                    // so adding an operator to the text run does not silently
                    // start reading the font size as a coordinate.
                    let head: Vec<&str> = line[..open].split_whitespace().collect();
                    if let Some(td) = head.iter().position(|t| *t == "Td") {
                        if td >= 2 {
                            if let (Ok(x), Ok(y)) =
                                (head[td - 2].parse::<f64>(), head[td - 1].parse::<f64>())
                            {
                                page.placed_texts.push((Pt(x), Pt(y), label.clone()));
                            }
                        }
                    }
                    page.texts.push(label);
                }
            }
            continue;
        }

        let tokens: Vec<&str> = line.split_whitespace().collect();
        match tokens.as_slice() {
            [x, y, "m"] => {
                current.clear();
                current.push((Pt(x.parse().unwrap()), Pt(y.parse().unwrap())));
            }
            [x, y, "l"] => current.push((Pt(x.parse().unwrap()), Pt(y.parse().unwrap()))),
            ["h"] => {}
            ["S"] => {
                if !current.is_empty() {
                    page.polylines.push(std::mem::take(&mut current));
                }
            }
            [x, y, w, h, "re", tail @ ..] => {
                let rect = (
                    x.parse().unwrap(),
                    y.parse().unwrap(),
                    w.parse().unwrap(),
                    h.parse().unwrap(),
                );
                if tail == ["W", "n"] {
                    page.clips.push(rect);
                } else {
                    page.rects.push(rect);
                }
            }
            _ => {}
        }
    }
    page
}
