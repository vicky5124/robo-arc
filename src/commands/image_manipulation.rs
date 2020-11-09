use image;
use photon_rs::{
    multiple::blend,
    native::open_image,
    transform::{resize, SamplingFilter},
    PhotonImage,
};

//use std::fs::File;
//use std::io::Write;
use std::borrow::Cow;
use std::sync::{Arc, Mutex, RwLock};

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    http::AttachmentType,
    model::channel::Message,
    prelude::Context,
};
use tokio::task::block_in_place;

async fn pride_image(
    image_vec: &[u8],
    name: String,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let mut og_image = PhotonImage::new_from_byteslice(image_vec.to_vec());
    let pride_path = format!("pride/{}.png", name);
    let mut pride_image = open_image(Box::leak(pride_path.into_boxed_str()))?;

    let og_image_arc = Arc::new(RwLock::new(&mut og_image));
    let og_image_clone = Arc::clone(&og_image_arc);
    let og_image_clone_clone = Arc::clone(&og_image_arc);

    block_in_place(move || {
        pride_image = {
            let og_x = og_image_clone_clone.read().unwrap().get_width();
            let og_y = og_image_clone_clone.read().unwrap().get_height();

            resize(&pride_image, og_x, og_y, SamplingFilter::Nearest)
        };

        let mut og_img_mut = og_image_clone.write().unwrap();

        blend(&mut og_img_mut, &pride_image, "overlay");
    });

    let mut result = Vec::new();

    let raw_pixels = og_image.get_raw_pixels();
    let width = og_image.get_width();
    let height = og_image.get_height();

    let img_buffer = image::ImageBuffer::from_vec(width, height, raw_pixels).unwrap();
    image::DynamicImage::ImageRgba8(img_buffer)
        .write_to(&mut result, image::ImageOutputFormat::Jpeg(255))?;

    //save_image(result, "lol_test_overlay.png");
    Ok(result)
}

async fn grayscale(image_vec: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    // Load the image as a buffer.
    let mut imgbuf = image::load_from_memory(&image_vec)?.into_rgba();

    let gray_bytes = Arc::new(Mutex::new(Vec::new()));
    let gray_bytes_clone = Arc::clone(&gray_bytes);

    // Iterate over the coordinates and pixels of the image
    // This makes the grading.
    block_in_place(move || {
        for (_, _, pixel) in imgbuf.enumerate_pixels_mut() {
            // Algorythm to transform RGB into black and white.
            // https://en.wikipedia.org/wiki/YIQ
            let r = (pixel.0[0] as f32 * 0.299 as f32).abs() as u8;
            let g = (pixel.0[1] as f32 * 0.587 as f32).abs() as u8;
            let b = (pixel.0[2] as f32 * 0.114 as f32).abs() as u8;

            let gray = r + g + b;

            *pixel = image::Rgba([gray, gray, gray, pixel.0[3]]);
        }

        // Save the image as “fractal.png”, the format is deduced from the path
        // imgbuf.save("grayscale.png")?;
        image::DynamicImage::ImageRgba8(imgbuf)
            .write_to(
                &mut *gray_bytes_clone.lock().unwrap(),
                image::ImageOutputFormat::Jpeg(255),
            )
            .expect("There was an error writing the image.");
    });

    let result = gray_bytes.lock().unwrap();
    Ok(result.clone().to_vec())
}

/// Grayscales the attached image.
/// - Currently only works with attached images, with a maximum size of 8k
///
/// Usage: `gray` and attach an image.
#[command]
#[aliases(grayscale)]
async fn gray(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    // obtains the first attachment on the message or None if the message doesn't have one.
    let first_attachment = &msg.attachments.get(0);
    let mut filename = &String::new();

    let (image_url, bytes) = match first_attachment {
        // if there was an attachment on the first possition, unwrap it.
        Some(x) => {
            // get the dimensions of the image.
            let dimensions = x.dimensions();

            // if the dimensions is None, it means it's not an image, but a normal file, so we respond acordingly.
            if dimensions == None {
                let err_message = "The provided file is not a valid image.".to_string();
                (err_message, vec![0])
            // else we download the image. Download returns a Result Vec<u8>
            } else {
                if dimensions.unwrap().0 > 7680 || dimensions.unwrap().1 > 4320 {
                    msg.reply(ctx, "The provided image is too large").await?;
                    return Ok(());
                }

                let bytes = x.download().await?;
                filename = &x.filename;

                //let mut file = File::create(filename)?;
                //file.write_all(&bytes)?;

                (x.url.to_owned(), bytes)
            }
        }
        // else say that an image was not provided.
        None => ("No image was provided.".to_string(), vec![0]),
    };

    // if an error was returned from the previous checks, say the error and finish the command.
    if bytes == vec![0] {
        msg.channel_id.say(ctx, image_url).await?;
        return Ok(());
    }

    // Uploads the grayscaled image bytes as an attachment
    // this is necessary to do as im never saving the image, just have the bytes as a vector.
    let grayscaled_bytes = grayscale(&bytes).await?;
    let attachment = AttachmentType::Bytes {
        data: Cow::from(grayscaled_bytes),
        filename: filename.to_owned(),
    };

    // Sends an embed with a link to the original image ~~and the prided image attached~~.
    msg.channel_id
        .send_message(ctx, |m| {
            m.add_file(attachment);
            m.embed(|e| {
                e.title("Original Image");
                e.url(image_url);
                e.image(format!("attachment://{}", filename));
                e
            });
            m
        })
        .await?;

    Ok(())
}

/// Prides the attached image.
/// - Currently only works with attached images, with a maximum size of 8k
///
/// Default: gay_gradient
/// Usage: `pride transgender_gradient` and attach an image.
///
/// Available flags:
/// **Agender**
/// - agender
/// - agender_gradient
///
/// **Asexual**
/// - asexual
/// - asexual_gradient
///
/// **Bisexual**
/// - bi
/// - bi_feminine
/// - bi_masculine
/// - bi_gradient
///
/// **Gay**
/// - gay
/// - gay_gradient
///
/// **Lesbian (2018)**
/// - lesbian
/// - lesbian_gradient
///
/// **Non-Binary**
/// - nonbinary
/// - nonbinary_gradient
///
/// **Pansexual**
/// - pan
/// - pan_feminine
/// - pan_masculine
/// - pan_gradient
///
/// **Transgender**
/// - transgender
/// - transgender_reverse
/// - transgender_gradient
#[command]
async fn pride(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let arg = if let Ok(x) = args.single::<String>() {
        x
    } else {
        "gay_gradient".to_string()
    };
    let first_attachment = &msg.attachments.get(0);
    let mut filename = &String::new();

    let (image_url, bytes) = match first_attachment {
        // if there was an attachment on the first possition, unwrap it.
        Some(x) => {
            // get the dimensions of the image.
            let dimensions = x.dimensions();

            // if the dimensions is None, it means it's not an image, but a normal file, so we respond acordingly.
            if dimensions == None {
                let err_message = "The provided file is not a valid image.".to_string();
                (err_message, vec![0])
            // else we download the image. Download returns a Result Vec<u8>
            } else {
                if dimensions.unwrap().0 > 7680 || dimensions.unwrap().1 > 4320 {
                    msg.reply(ctx, "The provided image is too large").await?;
                    return Ok(());
                }

                let bytes = x.download().await?;
                filename = &x.filename;

                //let mut file = File::create(filename)?;
                //file.write_all(&bytes)?;

                (x.url.to_owned(), bytes)
            }
        }
        // else say that an image was not provided.
        None => ("No image was provided.".to_string(), vec![0]),
    };

    // if an error was returned from the previous checks, say the error and finish the command.
    if bytes == vec![0] {
        msg.channel_id.say(ctx, image_url).await?;
        return Ok(());
    }

    // Uploads the grayscaled image bytes as an attachment
    // this is necessary to do as im never saving the image, just have the bytes as a vector.
    let prided_bytes = pride_image(&bytes, arg).await?;
    let attachment = AttachmentType::Bytes {
        data: Cow::from(prided_bytes),
        filename: filename.to_owned(),
    };

    // Sends an embed with a link to the original image ~~and the prided image attached~~.
    msg.channel_id
        .send_message(ctx, |m| {
            m.add_file(attachment);
            m.embed(|e| {
                e.title("Original Image");
                e.url(image_url);
                e.image(format!("attachment://{}", filename));
                e
            });
            m
        })
        .await?;

    Ok(())
}

/// Same as `pride`, but it grayscales the image before applying the filter.
#[command]
#[aliases(
    pridegray,
    pride_gray,
    pride_grayscale,
    pridegrayscale,
    pg,
    pride_g,
    prideg,
    pgray,
    p_gray,
    p_grayscale,
    pgrayscale
)]
async fn pride_pre_grayscaled(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let arg = if let Ok(x) = args.single::<String>() {
        x
    } else {
        "gay_gradient".to_string()
    };
    let first_attachment = &msg.attachments.get(0);
    let mut filename = &String::new();

    let (image_url, bytes) = match first_attachment {
        // if there was an attachment on the first possition, unwrap it.
        Some(x) => {
            // get the dimensions of the image.
            let dimensions = x.dimensions();

            // if the dimensions is None, it means it's not an image, but a normal file, so we respond acordingly.
            if dimensions == None {
                let err_message = "The provided file is not a valid image.".to_string();
                (err_message, vec![0])
            // else we download the image. Download returns a Result Vec<u8>
            } else {
                if dimensions.unwrap().0 > 7680 || dimensions.unwrap().1 > 4320 {
                    msg.reply(ctx, "The provided image is too large").await?;
                    return Ok(());
                }

                let bytes = x.download().await?;
                filename = &x.filename;

                //let mut file = File::create(filename)?;
                //file.write_all(&bytes)?;

                (x.url.to_owned(), bytes)
            }
        }
        // else say that an image was not provided.
        None => ("No image was provided.".to_string(), vec![0]),
    };

    // if an error was returned from the previous checks, say the error and finish the command.
    if bytes == vec![0] {
        msg.channel_id.say(ctx, image_url).await?;
        return Ok(());
    }

    // Uploads the grayscaled image bytes as an attachment
    // this is necessary to do as im never saving the image, just have the bytes as a vector.
    let grayscaled_bytes = grayscale(&bytes).await?;
    let prided_bytes = pride_image(&grayscaled_bytes, arg).await?;
    let attachment = AttachmentType::Bytes {
        data: Cow::from(prided_bytes),
        filename: filename.to_owned(),
    };

    // Sends an embed with a link to the original image ~~and the prided image attached~~.
    msg.channel_id
        .send_message(ctx, |m| {
            m.add_file(attachment);
            m.embed(|e| {
                e.title("Original Image");
                e.url(image_url);
                e.image(format!("attachment://{}", filename));
                e
            });
            m
        })
        .await?;

    Ok(())
}
