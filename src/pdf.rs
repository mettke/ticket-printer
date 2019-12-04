use crate::{config::Config, services::Ticket, Result};
use failure::ResultExt;
use image::{Luma, Pixel};
use pdf_canvas::{BuiltinFont, FontSource};
use pdfpdf::{Alignment, Font, Image, Pdf, Point, Size};
use qrcode::{EcLevel, QrCode, Version};
use std::{
    path::{Path, PathBuf},
    process::Command,
};
use tempfile::tempdir;
use urlshortener::{client::UrlShortener, providers::Provider};

pub fn print_tickets(
    config: &Config,
    tickets: &mut Vec<Ticket>,
) -> Result<()> {
    if tickets.is_empty() {
        println!("No tickets marked for printing.");
        return Ok(());
    }
    let (tmp_dir, pdf_path) = if let Some(out_dir) =
        config.global.as_ref().and_then(|g| g.out_dir.as_ref())
    {
        (None, PathBuf::from(out_dir))
    } else {
        let dir = tempdir().with_context(|_| {
            "Could not create temporary file for pdf".to_string()
        })?;
        let path = dir.path().to_path_buf();
        (Some(dir), path)
    };
    if config.printer.is_none() {
        println!(
            "Missing printer configuration. Only saving pdfs."
        );
    }
    while !tickets.is_empty() {
        if let Some(mut ticket) = tickets.last_mut() {
            minify_url_if_possible(&mut ticket);
            let pdf = create_pdf(config, &pdf_path, ticket)?;
            print_pdf(config, &pdf)?;
            println!("Printed: {}", ticket.id);
        }
        let _ = tickets.pop();
    }
    if let Some(tmp_dir) = tmp_dir {
        drop(tmp_dir);
    }
    Ok(())
}

fn minify_url_if_possible(ticket: &mut Ticket) {
    if let Ok(shortener) = UrlShortener::new() {
        let providers = [
            Provider::BamBz,
            Provider::Bmeo,
            Provider::FifoCc,
            Provider::HmmRs,
            Provider::IsGd,
            Provider::SCoop,
            Provider::TinyPh,
            Provider::TnyIm,
            Provider::UrlShortenerIo,
            Provider::VGd,
        ];
        for provider in &providers {
            if let Ok(short_url_res) =
                shortener.generate(&ticket.url, provider)
            {
                ticket.url = short_url_res;
                break;
            }
        }
    }
}

fn create_pdf(
    config: &Config,
    pdf_dir: &Path,
    ticket: &Ticket,
) -> Result<PathBuf> {
    let pdf_path = pdf_dir.join(format!("{}.pdf", ticket.id));
    let mut pdf = Pdf::new();
    let _ = pdf.add_page(Size {
        width: config.pdf.width,
        height: config.pdf.height,
    });

    let width = (config.pdf.width / 2.0)
        - config.pdf.margin
        - (config.pdf.qrcode_seperator_margin / 2.0);
    let height = (config.pdf.height / 2.0)
        - config.pdf.margin
        - (config.pdf.title_seperator_margin / 2.0);
    if let Some(qrcode) = setup_qrcode(ticket, width, height) {
        let dim = qrcode.dimensions();
        let mut image_bytes: Vec<u8> =
            Vec::with_capacity((dim.0 * dim.1 * 3) as usize);
        for (_, _, pixel) in qrcode.enumerate_pixels() {
            let [r, g, b]: [u8; 3] = pixel.to_rgb().0;
            image_bytes.push(r);
            image_bytes.push(g);
            image_bytes.push(b);
        }
        let image = Image::new(&image_bytes, dim.0, dim.1);
        let mut center_qrcode_y = (height - dim.1 as f32) / 2.0;
        if 0.0 > center_qrcode_y {
            center_qrcode_y = 0.0;
        }
        let _ = pdf.add_image_at(
            image,
            Point {
                x: config.pdf.margin,
                y: config.pdf.margin + center_qrcode_y,
            },
        );
    }
    setup_titel(config, ticket, &mut pdf);
    pdf.font(Font::Helvetica, config.pdf.subtitle_size)
        .draw_text(
            Point {
                x: config.pdf.width - config.pdf.margin,
                y: config.pdf.margin,
            },
            Alignment::BottomRight,
            &ticket.subtitel,
        )
        .write_to(pdf_path.clone())
        .with_context(|_| {
            "could not create pdf file".to_string()
        })?;
    Ok(pdf_path)
}

fn setup_titel(config: &Config, ticket: &Ticket, pdf: &mut Pdf) {
    let text_width = config.pdf.width - (2.0 * config.pdf.margin);
    let text_height = (config.pdf.height / 2.0)
        - config.pdf.margin
        - (config.pdf.title_seperator_margin / 2.0);
    let fontsize = text_height / (config.pdf.title_lines as f32);

    let words = ticket.titel.split_whitespace();
    let mut current_line = String::new();
    let mut line = 0;
    let font = BuiltinFont::Helvetica;
    let _ = pdf.font(Font::Helvetica, fontsize);
    for mut word in words {
        let word_whitespace = format!(" {}", word);
        if !current_line.is_empty() {
            word = &word_whitespace;
        }
        while line < config.pdf.title_lines {
            if font.get_width(
                fontsize,
                &format!("{}{}", current_line, word),
            ) < text_width
            {
                current_line.push_str(word);
                break;
            } else if current_line.is_empty() {
                for c in word.chars() {
                    let cur_width = font.get_width(
                        fontsize,
                        &format!("{}{}", current_line, c),
                    );
                    if cur_width < text_width {
                        current_line.push(c);
                    } else {
                        break;
                    }
                }
                break;
            } else {
                let _ = pdf.draw_text(
                    Point {
                        x: config.pdf.width / 2.0,
                        y: config.pdf.height
                            - line as f32 * fontsize,
                    },
                    Alignment::TopCenter,
                    &current_line,
                );
                line += 1;
                current_line = String::new();
            }
        }
        if line >= config.pdf.title_lines {
            break;
        }
    }
    if line < config.pdf.title_lines {
        let _ = pdf.draw_text(
            Point {
                x: config.pdf.width / 2.0,
                y: config.pdf.height - line as f32 * fontsize,
            },
            Alignment::TopCenter,
            &current_line,
        );
    }
}

fn setup_qrcode(
    ticket: &Ticket,
    width: f32,
    height: f32,
) -> Option<image::ImageBuffer<Luma<u8>, Vec<u8>>> {
    QrCode::with_version(
        &ticket.url,
        Version::Normal(3),
        EcLevel::L,
    )
    .map(|qrcode| {
        qrcode
            .render::<Luma<u8>>()
            .quiet_zone(false)
            .max_dimensions(width as u32, height as u32)
            .build()
    })
    .map_err(|e| {
        eprintln!("WARN: {}", e);
        e
    })
    .ok()
}

fn print_pdf(config: &Config, pdf: &Path) -> Result<()> {
    if let Some(printer) = &config.printer {
        let status = Command::new("/usr/bin/lp")
            .args(&[
            "-o",
            "fit-to-page",
            "-o",
            &format!("media={}", printer.media),
            "-o",
            &printer.orientation,
            "-n",
            &printer.number_of_copies.to_string(),
            "-d",
            &printer.name,
            pdf.to_str().unwrap_or(""),
            ])
            .status()
            .with_context(|_| {
                "Unable to execute /usr/bin/lp. Is lp installed?"
                    .to_string()
            })?;
        if status.success() {
            Ok(())
        } else {
            Err(failure::err_msg(
                "Unable to print ticket. LP failed with non zero",
            )
            .into())
        }
    } else {
        Ok(())
    }
}
