use std::collections::HashMap;
// use std::fs::OpenOptions;
// use std::io::Write;

use image::io::Reader;
use image::{DynamicImage, GenericImageView, GrayImage, Pixel, Rgba, RgbaImage};
use num_complex::Complex;
use rustfft::FftPlanner;

// 读取图片
pub fn reading_image(path: &str) -> DynamicImage {
    let image = match Reader::open(path) {
        Ok(image) => image,
        Err(_) => {
            println!("未能找到图片，请检查图片路径");
            panic!();
        }
    };

    let image = image.decode().unwrap();

    // 打印图片信息
    println!("图片尺寸：{:?}", image.dimensions());
    println!("图片颜色模式：{:?}", image.color());
    println!("图片位深度：{:?}", image.color().bits_per_pixel());

    image
}

// 图像灰度变换
pub fn image_to_gray(image: &DynamicImage) -> Vec<u8> {
    let image_luma: Vec<u8> = image
        .pixels()
        .map(|(_, _, pixel)| {
            let r = pixel[0] as f32;
            let g = pixel[1] as f32;
            let b = pixel[2] as f32;

            let gray = 0.299 * r + 0.587 * g + 0.114 * b;
            let gray = gray.round() as u8;
            gray
        })
        .collect();

    image_luma
}

// 直方图均值化处理
pub fn image_gray_average(image: &DynamicImage) -> Vec<u8> {
    let image_luma = image_to_gray(&image);

    let mut gray_data: HashMap<u8, i32> = HashMap::new();
    for v in &image_luma {
        gray_data
            .entry(*v)
            .and_modify(|counter| *counter += 1)
            .or_insert(1);
    }

    let mut gray_data: Vec<(u8, i32)> = gray_data.iter().map(|(k, v)| (*k, *v)).collect();
    gray_data.sort_by(|a, b| a.0.cmp(&b.0));

    // // 输出灰度数据
    // for i in &gray_data {
    //     let mut f = OpenOptions::new()
    //         .write(true)
    //         .create(true)
    //         .append(true)
    //         .open("gray_data.txt")
    //         .unwrap();

    //     f.write_all(format!("({} {})\n", i.0, i.1).as_bytes())
    //         .unwrap();
    // }

    let mut sum: f32 = 0.0;
    let mut temp: f32 = 0.0;
    let n = image.dimensions().0 * image.dimensions().1;
    let gray_data: Vec<(u8, u8)> = gray_data
        .iter_mut()
        .map(|(k, v)| {
            let v = *v as f32;
            let n = n as f32;
            sum as f32;
            sum = temp;
            sum = sum + v / n;
            temp = sum;

            sum = (sum * 255.0 + 0.5).floor();
            let sum = sum as u8;
            (*k, sum)
        })
        .collect();

    // // 输出灰度均值化数据
    // for i in &gray_data {
    //     let mut f = OpenOptions::new()
    //         .write(true)
    //         .create(true)
    //         .append(true)
    //         .open("gray_average_data.txt")
    //         .unwrap();

    //     f.write_all(format!("({} {})\n", i.0, i.1).as_bytes())
    //         .unwrap();
    // }

    let map: HashMap<_, _> = gray_data.into_iter().collect();

    let image_luma: Vec<u8> = image_luma
        .iter()
        .map(|v| {
            let v = map.get_key_value(v).unwrap().1.clone();
            v
        })
        .collect();

    image_luma
}

// 图像输出
pub fn image_output(image: &DynamicImage, file: &str) {
    image.save(file).unwrap();
}

// // 获取图片RGBA数组并写入txt文件
// pub fn image_output_rgba(image: DynamicImage, file: &str) {
//     for (x, y, pixel) in image.pixels() {
//         let mut f = OpenOptions::new()
//             .write(true)
//             .create(true)
//             .append(true)
//             .open(file)
//             .unwrap();

//         f.write_all(format!("x: {}, y: {}, rgba: {:?}\n", x, y, pixel).as_bytes())
//             .unwrap();
//     }
// }

// 灰度线性变换
pub fn gray_linear_transfromationg(
    image: &DynamicImage,
    a: f32,
    b: f32,
    gray_level: u8,
) -> Vec<u8> {
    let gray_level = gray_level as f32;

    let new_image_luma: Vec<u8> = image_to_gray(&image)
        .iter()
        .map(|v| {
            let v = *v as f32;
            let mut gray = a * v + b;
            if gray > gray_level {
                gray = gray_level
            }

            if gray < 0.0 {
                gray = 0.0
            }

            gray as u8
        })
        .collect();

    new_image_luma
}

// 灰度图快速傅里叶变换
pub fn gray_fast_fourier_transform(image: &DynamicImage) -> GrayImage {
    let width = image.width() as usize;
    let height = image.height() as usize;
    let gray_image = image_to_gray(image);
    let gray_image: Vec<f64> = gray_image.into_iter().map(|x| x as f64).collect();

    let mut buffer: Vec<Vec<Complex<f64>>> = vec![vec![Complex::new(0.0, 0.0); width]; height];
    for (i, v) in gray_image.into_iter().enumerate() {
        let row = i / width;
        let col = i % width;
        let value = v * (-1_f64).powf(row as f64 + col as f64);
        buffer[row][col] = Complex::from(value);
    }

    let fft = FftPlanner::new().plan_fft_forward(width);
    for i in buffer.iter_mut() {
        fft.process(i);
    }

    let mut temp = vec![vec![Complex::new(0.0, 0.0); height]; width];
    for i in 0..height {
        for j in 0..width {
            temp[width - 1 - j][i] = buffer[i][j];
        }
    }
    drop(buffer);

    let fft = FftPlanner::new().plan_fft_forward(height);
    for i in temp.iter_mut() {
        fft.process(i);
    }

    let mut buffer: Vec<Vec<Complex<f64>>> = vec![vec![Complex::new(0.0, 0.0); width]; height];
    for i in 0..width {
        for j in 0..height {
            buffer[j][width - 1 - i] = temp[i][j];
        }
    }
    drop(temp);

    let mut buffer: Vec<Complex<f64>> = buffer.into_iter().flatten().collect();
    let buffer: Vec<f64> = buffer.iter_mut().map(|x| x.norm()).collect();

    let max = buffer
        .iter()
        .fold(0.0, |max, x| if max > *x { max } else { *x });
    let max = 50000_f64;
    let min = buffer
        .iter()
        .fold(100000000.0, |min, x| if min < *x { min } else { *x });
    println!("max: {}, min: {}", max, min);

    let buffer: Vec<u8> = buffer
        .iter()
        .map(|x| ((x - min) / (max - min) * 255.0) as u8)
        .collect();

    let image: GrayImage = GrayImage::from_vec(width as u32, height as u32, buffer).unwrap();
    image
}

// 图像对比度变换
pub fn adjust_contrast_image(image: &DynamicImage, contrast: f32) -> DynamicImage {
    let new_image = image.adjust_contrast(contrast);
    new_image
}

// 图像亮度变换
pub fn adjust_brightness_image(image: &DynamicImage, brightness: i32) -> DynamicImage {
    let new_image = image.brighten(brightness);
    new_image
}

// 图像高斯模糊
pub fn blur_image(image: &DynamicImage, sigma: f32) -> DynamicImage {
    let new_image = image.blur(sigma);
    new_image
}

// 图像平滑
pub fn smooth_image(image: &DynamicImage, radius: u32) -> DynamicImage {
    let width = image.width();
    let height = image.height();

    let mut buffer: Vec<Vec<Rgba<u8>>> =
        vec![vec![Rgba([0, 0, 0, 0]); width as usize]; height as usize];
    for (_, v) in image.pixels().enumerate() {
        buffer[v.1 as usize][v.0 as usize] = v.2;
    }

    let mut new_buffer = buffer.clone();

    for i in radius..height - radius {
        for j in radius..width - radius {
            let mut list = Vec::new();

            for a in i - radius..i + radius + 1 {
                for b in j - radius..j + radius + 1 {
                    list.push(buffer[a as usize][b as usize]);
                }
            }

            let mut red_list = Vec::new();
            let mut green_list = Vec::new();
            let mut blue_list = Vec::new();

            for v in list.iter() {
                red_list.push(v.0[0]);
                green_list.push(v.0[1]);
                blue_list.push(v.0[2]);
            }

            red_list.sort();
            green_list.sort();
            blue_list.sort();

            let pixel = Rgba([
                red_list[red_list.len() / 2],
                green_list[green_list.len() / 2],
                blue_list[blue_list.len() / 2],
                255,
            ]);

            new_buffer[i as usize][j as usize] = pixel;
        }
    }

    let new_buffer: Vec<Rgba<u8>> = new_buffer.into_iter().flatten().collect();
    let mut image = RgbaImage::new(width, height);

    for (k, v) in image.pixels_mut().enumerate() {
        *v = new_buffer[k];
    }

    let image = image::DynamicImage::ImageRgba8(image);
    image
}

// 图像锐化
pub fn sharpen_image(image: &DynamicImage, index: i32) -> DynamicImage {
    let width = image.width();
    let height = image.height();

    let mut buffer: Vec<Vec<Rgba<u8>>> =
        vec![vec![Rgba([0, 0, 0, 0]); width as usize]; height as usize];
    for (_, v) in image.pixels().enumerate() {
        buffer[v.1 as usize][v.0 as usize] = v.2;
    }

    let mut new_buffer = buffer.clone();
    let laplace_filter = [1, 1, 1, 1, -8, 1, 1, 1, 1];

    for i in 1..height - 1 {
        for j in 1..width - 1 {
            let mut list = Vec::new();

            for a in i - 1..i + 2 {
                for b in j - 1..j + 2 {
                    list.push(buffer[a as usize][b as usize]);
                }
            }

            let mut red_list = Vec::new();
            let mut green_list = Vec::new();
            let mut blue_list = Vec::new();

            for v in list.iter() {
                red_list.push(v.0[0]);
                green_list.push(v.0[1]);
                blue_list.push(v.0[2]);
            }

            let mut value: [i32; 3] = [0, 0, 0];

            for (k, v) in laplace_filter.iter().enumerate() {
                value[0] = value[0] + v * red_list[k] as i32;
                value[1] = value[1] + v * green_list[k] as i32;
                value[2] = value[2] + v * blue_list[k] as i32;
            }

            let pixel = buffer[i as usize][j as usize].channels();
            for (k, v) in value.iter_mut().enumerate() {
                *v = *v * index + pixel[k] as i32;
            }

            let pixel = Rgba([
                value[0].clamp(0, 255) as u8,
                value[1].clamp(0, 255) as u8,
                value[2].clamp(0, 255) as u8,
                255,
            ]);

            new_buffer[i as usize][j as usize] = pixel;
        }
    }

    let new_buffer: Vec<Rgba<u8>> = new_buffer.into_iter().flatten().collect();
    let mut image = RgbaImage::new(width, height);

    for (k, v) in image.pixels_mut().enumerate() {
        *v = new_buffer[k];
    }

    let image = image::DynamicImage::ImageRgba8(image);
    image
}

// 图像分割
pub fn segmentate_image(image: &DynamicImage, index: u8) -> GrayImage {
    let width = image.width();
    let height = image.height();

    let mut image = image_to_gray(image);

    for i in image.iter_mut(){
        if *i > index {
            *i = 255
        }else{
            *i = 0
        }
    }

    let image: GrayImage = GrayImage::from_vec(width, height, image).unwrap();
    image
}