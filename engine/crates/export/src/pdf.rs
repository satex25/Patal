//! A minimal, deterministic PDF writer.
//!
//! Roughly two per cent of the PDF specification: straight lines, a
//! rectangular clip, dashes, and text in a base-14 font. That is the whole
//! surface a flat pattern needs.
//!
//! It is hand-rolled for one reason, and it is not dependency politics. The
//! golden-file test in this crate compares bytes, and a byte comparison is
//! only worth having if the bytes are a function of the geometry alone. Every
//! general-purpose PDF crate stamps a creation date, or compresses streams
//! through a zlib whose exact output is not a stability guarantee, or both —
//! and a golden test that goes red on an unrelated dependency bump is a test
//! that gets re-blessed unread at two in the morning, which is worse than not
//! having one.
//!
//! So: no `/CreationDate` (there is no `/Info` dictionary at all), a pinned
//! `/ID`, no compression, and one number formatter. The same pieces produce
//! the same bytes on any machine, this year and next.
//!
//! ## Units
//!
//! Everything in here is [`Pt`]. Conversion from millimetres happens before
//! anything reaches this module, and the content stream never contains a
//! `cm` scale matrix. That is deliberate: under a millimetre-scaled CTM, a
//! `1 w` line width would come out a millimetre thick instead of a point, and
//! a stroke is centred on its path — so a 1mm line is half a millimetre of
//! ambiguity on each side about where the scissors go. With an identity CTM,
//! a width in points is a width in points.

use std::fmt::Write as _;

use crate::units::Pt;

/// Stroke width for everything this crate draws, in points, on an identity
/// CTM.
///
/// 0.25pt is about 0.09mm — well inside the 0.4mm the geometry crate treats
/// as cutter tolerance, so which side of the line the scissors take cannot
/// matter. ADR-008 records the convention that the true line is the centre of
/// the stroke.
pub const HAIRLINE_PT: f64 = 0.25;

/// One page: its media box and its content stream.
pub struct Page {
    pub width: Pt,
    pub height: Pt,
    pub canvas: Canvas,
}

/// A content stream under construction.
#[derive(Default)]
pub struct Canvas {
    ops: String,
}

/// Formats a number for a content stream, identically on every platform.
///
/// Four decimal places is about a ten-thousandth of a point — a hundred times
/// finer than any printer can place ink, and finite, so the output cannot
/// depend on how a particular libc renders the seventeenth significant digit.
/// Trailing zeros are trimmed because the golden file is read by humans when
/// it fails.
fn num(value: f64) -> String {
    let mut s = format!("{value:.4}");
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    // `-0` and `0` are the same number and must be the same bytes.
    if s == "-0" {
        s = "0".to_string();
    }
    s
}

/// Encodes a label for a base-14 font with `WinAnsiEncoding`.
///
/// Characters outside that encoding become `?`. This is a real limitation and
/// it is worth being explicit about where it bites: a piece named `Pātāl`
/// prints as `P?t?l`. The alternative is embedding a font subset, which is an
/// order of magnitude more code than everything else in this module put
/// together, for a gain that is entirely typographic — no label in this
/// document is load-bearing. Nothing a person cuts along depends on it. If
/// piece names in Devanagari or with macrons become common, embed a font;
/// until then this is documented and tested rather than silent.
fn winansi(text: &str) -> Vec<u8> {
    text.chars()
        .map(|c| {
            let code = c as u32;
            if (0x20..=0x7E).contains(&code) || (0xA0..=0xFF).contains(&code) {
                code as u8
            } else {
                b'?'
            }
        })
        .collect()
}

/// Escapes an encoded label into a PDF literal string.
fn pdf_string(text: &str) -> String {
    let mut out = String::from("(");
    for byte in winansi(text) {
        match byte {
            b'(' | b')' | b'\\' => {
                out.push('\\');
                out.push(byte as char);
            }
            0x20..=0x7E => out.push(byte as char),
            // Octal escape keeps the stream pure ASCII, so the file stays
            // readable in a text editor and diffable when the golden fails.
            other => {
                let _ = write!(out, "\\{other:03o}");
            }
        }
    }
    out.push(')');
    out
}

impl Canvas {
    pub fn new() -> Self {
        Self::default()
    }

    /// A comment line in the content stream.
    ///
    /// Not decoration. The tile-reassembly test reads these back to learn
    /// which model coordinate each page's window starts at, so the assertion
    /// that the sheets tile correctly is made against what the document
    /// actually says rather than against the code that wrote it. They also
    /// mean a person can open the PDF in a text editor and see which sheet is
    /// which.
    pub fn comment(&mut self, text: &str) -> &mut Self {
        // A newline inside a comment would end it and turn the rest into
        // operators. Replaced rather than rejected: a comment is not a cut
        // line, and there is no caller that can act on the error.
        let _ = writeln!(self.ops, "% {}", text.replace(['\r', '\n'], " "));
        self
    }

    pub fn save(&mut self) -> &mut Self {
        self.ops.push_str("q\n");
        self
    }

    pub fn restore(&mut self) -> &mut Self {
        self.ops.push_str("Q\n");
        self
    }

    /// Clips everything that follows to a rectangle.
    ///
    /// This is how tiling works: each sheet draws the piece's whole outline
    /// in its own translated coordinates and lets the clip decide what shows.
    /// The alternative — cutting the polygon at the tile edge in Rust — is
    /// geometry, and geometry that decides where a line stops is exactly the
    /// second cut-line implementation constraint C11 exists to forbid.
    pub fn clip_rect(&mut self, x: Pt, y: Pt, width: Pt, height: Pt) -> &mut Self {
        let _ = writeln!(
            self.ops,
            "{} {} {} {} re W n",
            num(x.get()),
            num(y.get()),
            num(width.get()),
            num(height.get())
        );
        self
    }

    /// Stroke width, in points, on an identity CTM. See [`HAIRLINE_PT`].
    pub fn line_width_pt(&mut self, width: f64) -> &mut Self {
        let _ = writeln!(self.ops, "{} w", num(width));
        self
    }

    /// Grey level, 0 black to 1 white.
    pub fn grey(&mut self, level: f64) -> &mut Self {
        let _ = writeln!(self.ops, "{} G", num(level));
        self
    }

    /// Dash pattern in points; an empty slice restores a solid line.
    pub fn dash(&mut self, pattern: &[f64]) -> &mut Self {
        let joined: Vec<String> = pattern.iter().map(|v| num(*v)).collect();
        let _ = writeln!(self.ops, "[{}] 0 d", joined.join(" "));
        self
    }

    /// Strokes a polyline. `close` joins the last point back to the first.
    pub fn polyline(&mut self, points: &[(Pt, Pt)], close: bool) -> &mut Self {
        let Some(((first_x, first_y), rest)) = points.split_first() else {
            return self;
        };
        let _ = writeln!(self.ops, "{} {} m", num(first_x.get()), num(first_y.get()));
        for (x, y) in rest {
            let _ = writeln!(self.ops, "{} {} l", num(x.get()), num(y.get()));
        }
        if close {
            self.ops.push_str("h\n");
        }
        self.ops.push_str("S\n");
        self
    }

    pub fn rect(&mut self, x: Pt, y: Pt, width: Pt, height: Pt) -> &mut Self {
        let _ = writeln!(
            self.ops,
            "{} {} {} {} re S",
            num(x.get()),
            num(y.get()),
            num(width.get()),
            num(height.get())
        );
        self
    }

    pub fn line(&mut self, from: (Pt, Pt), to: (Pt, Pt)) -> &mut Self {
        self.polyline(&[from, to], false)
    }

    /// Draws text with its baseline at `(x, y)`, in Helvetica at `size_pt`.
    pub fn text(&mut self, x: Pt, y: Pt, size_pt: f64, label: &str) -> &mut Self {
        let _ = writeln!(
            self.ops,
            "BT /F1 {} Tf {} {} Td {} Tj ET",
            num(size_pt),
            num(x.get()),
            num(y.get()),
            pdf_string(label)
        );
        self
    }

    pub fn as_str(&self) -> &str {
        &self.ops
    }
}

/// A constant file identifier.
///
/// PDF wants an `/ID` that distinguishes one document from another, and every
/// producer generates it from the clock or a random source. Both make the
/// bytes of an export depend on when it ran, which would make the golden test
/// impossible and, more to the point, would mean two exports of the same
/// pattern are not obviously the same file. Pinning it says plainly: this
/// document is a pure function of the pieces that went into it.
const PINNED_ID: &str = "50617461-6c2d-4578-706f-72742d76310a";

/// Serialises pages into a complete PDF file.
pub fn render(pages: &[Page]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    // Objects are numbered from 1; `offsets[i]` is the byte offset of object
    // `i + 1`, which the cross-reference table needs exactly right — a PDF
    // with a wrong xref opens in some readers and not others, which is the
    // worst of the available failure modes.
    let mut offsets: Vec<usize> = Vec::new();

    out.extend_from_slice(b"%PDF-1.7\n");
    // The conventional high-bit comment that marks the file as binary, so
    // transfer tools do not "helpfully" translate line endings inside it.
    out.extend_from_slice(b"%\xE2\xE3\xCF\xD3\n");
    out.extend_from_slice(b"% Written by Patal. True scale: 1mm in the model is 1mm on paper.\n");

    let page_object_id = |index: usize| 4 + 2 * index;
    let content_object_id = |index: usize| 5 + 2 * index;

    let begin_object = |out: &mut Vec<u8>, offsets: &mut Vec<usize>, id: usize| {
        debug_assert_eq!(offsets.len() + 1, id, "objects must be written in order");
        offsets.push(out.len());
        out.extend_from_slice(format!("{id} 0 obj\n").as_bytes());
    };

    begin_object(&mut out, &mut offsets, 1);
    out.extend_from_slice(b"<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    begin_object(&mut out, &mut offsets, 2);
    let kids: Vec<String> = (0..pages.len())
        .map(|i| format!("{} 0 R", page_object_id(i)))
        .collect();
    out.extend_from_slice(
        format!(
            "<< /Type /Pages /Kids [{}] /Count {} >>\nendobj\n",
            kids.join(" "),
            pages.len()
        )
        .as_bytes(),
    );

    begin_object(&mut out, &mut offsets, 3);
    out.extend_from_slice(
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>\nendobj\n",
    );

    for (index, page) in pages.iter().enumerate() {
        begin_object(&mut out, &mut offsets, page_object_id(index));
        out.extend_from_slice(
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}] \
                 /Resources << /Font << /F1 3 0 R >> >> /Contents {} 0 R >>\nendobj\n",
                num(page.width.get()),
                num(page.height.get()),
                content_object_id(index)
            )
            .as_bytes(),
        );

        let stream = page.canvas.as_str().as_bytes();
        begin_object(&mut out, &mut offsets, content_object_id(index));
        out.extend_from_slice(format!("<< /Length {} >>\nstream\n", stream.len()).as_bytes());
        out.extend_from_slice(stream);
        out.extend_from_slice(b"endstream\nendobj\n");
    }

    let xref_offset = out.len();
    let size = offsets.len() + 1;
    out.extend_from_slice(format!("xref\n0 {size}\n").as_bytes());
    out.extend_from_slice(b"0000000000 65535 f \n");
    for offset in &offsets {
        out.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {size} /Root 1 0 R /ID [<{PINNED_ID}> <{PINNED_ID}>] >>\n\
             startxref\n{xref_offset}\n%%EOF\n"
        )
        .as_bytes(),
    );

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_are_formatted_the_same_way_every_time() {
        assert_eq!(num(1.0), "1");
        assert_eq!(num(1.5), "1.5");
        assert_eq!(num(141.732_283_464_566_93), "141.7323");
        assert_eq!(num(-0.000_001), "0", "negative zero must print as zero");
        assert_eq!(num(0.0), "0");
        assert_eq!(num(-12.25), "-12.25");
    }

    #[test]
    fn labels_are_escaped_so_a_piece_name_cannot_break_the_stream() {
        assert_eq!(pdf_string("Front (left)"), "(Front \\(left\\))");
        assert_eq!(pdf_string("back\\panel"), "(back\\\\panel)");
    }

    #[test]
    fn characters_outside_winansi_become_question_marks_visibly() {
        // Documented, not silent: see `winansi`. A label is not a cut line.
        assert_eq!(pdf_string("Pātāl"), "(P?t?l)");
        // Latin-1 survives, which covers most European piece names.
        assert_eq!(pdf_string("Épaule"), "(\\311paule)");
    }

    #[test]
    fn a_rendered_file_has_a_wellformed_trailer_and_xref() {
        let mut canvas = Canvas::new();
        canvas
            .line_width_pt(HAIRLINE_PT)
            .rect(Pt(10.0), Pt(10.0), Pt(100.0), Pt(100.0));
        let bytes = render(&[Page {
            width: Pt(595.2756),
            height: Pt(841.8898),
            canvas,
        }]);
        let text = String::from_utf8_lossy(&bytes);

        assert!(bytes.starts_with(b"%PDF-1.7\n"));
        assert!(bytes.ends_with(b"%%EOF\n"));
        assert!(!text.contains("/CreationDate"), "no clock in the bytes");
        assert!(
            text.contains("/Size 6"),
            "catalog, pages, font, page, stream"
        );

        // The xref offset in the trailer must actually point at `xref`.
        //
        // Read out of the *bytes*, not out of a lossy UTF-8 view of them: the
        // binary marker comment in the header is invalid UTF-8, and lossy
        // decoding replaces its four bytes with a three-byte replacement
        // character. Every offset after that point would then be one byte
        // short — a test that fails on a correct file, which is the kind of
        // test that gets deleted rather than believed.
        let tail = String::from_utf8_lossy(&bytes[bytes.len() - 64..]).to_string();
        let startxref: usize = tail
            .rsplit("startxref\n")
            .next()
            .unwrap()
            .lines()
            .next()
            .unwrap()
            .trim()
            .parse()
            .expect("startxref is a number");
        assert!(
            bytes[startxref..].starts_with(b"xref\n"),
            "the trailer's startxref must point at the cross-reference table"
        );

        // And every entry in that table must point at the object it claims.
        for (index, line) in String::from_utf8_lossy(&bytes[startxref..])
            .lines()
            .skip(3)
            .take(5)
            .enumerate()
        {
            let Some(offset) = line.split_whitespace().next() else {
                continue;
            };
            let offset: usize = offset.parse().expect("an xref offset is a number");
            // Line 0 after the header, the object count and the free entry is
            // object 1.
            let expected = format!("{} 0 obj", index + 1);
            assert!(
                bytes[offset..].starts_with(expected.as_bytes()),
                "xref entry for object {} points at the wrong place",
                index + 1
            );
        }
    }

    #[test]
    fn the_same_input_renders_to_the_same_bytes() {
        let build = || {
            let mut canvas = Canvas::new();
            canvas.text(Pt(10.0), Pt(10.0), 8.0, "sheet 1");
            render(&[Page {
                width: Pt(595.2756),
                height: Pt(841.8898),
                canvas,
            }])
        };
        assert_eq!(build(), build());
    }
}
