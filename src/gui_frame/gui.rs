use eframe::{egui, egui::IconData};
use image::{self, GenericImageView, GrayImage};
use rfd::FileDialog;
use std::sync::Arc;

use crate::image_util::*;

use self::image_util::*;

pub fn run() {
    // 创建窗口默认配置
    let mut native_options = eframe::NativeOptions::default();

    // 设置窗口图标
    let icon_data = include_bytes!("../image/icon/icon.png");
    let img = image::load_from_memory_with_format(icon_data, image::ImageFormat::Png).unwrap();
    let rgba_data = img.into_rgba8();
    let (width, height) = (rgba_data.width(), rgba_data.height());
    let rgba: Vec<u8> = rgba_data.into_raw();
    native_options.viewport.icon = Some(Arc::<IconData>::new(IconData {
        rgba,
        width,
        height,
    }));

    eframe::run_native(
        "Image App",
        native_options,
        Box::new(|cc| Box::new(ImageApp::new(cc))),
    )
    .unwrap();
}

// 加载字体
fn load_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(include_bytes!("../Fonts/msyhl.ttc")),
    );
    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "my_font".to_owned());
    fonts
        .families
        .get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .push("my_font".to_owned());
    ctx.set_fonts(fonts);
}

struct ImageApp {
    string_values: Vec<String>,
    frame_count: u32,
    image_file_path: String,
    image_data: image::DynamicImage,
    state: Vec<bool>,
    line_transform: [i32; 2],
    adjust_contrast_value: f32,
    brightness_value: i32,
    blur_value: f32,
    segmentation_index: u8,
}

impl ImageApp {
    fn new(cc: &eframe::CreationContext) -> Self {
        load_fonts(&cc.egui_ctx);
        egui_extras::install_image_loaders(&cc.egui_ctx);
        Self {
            string_values: vec![
                "图片处理".to_string(),       //0
                "打开图片".to_string(),       //1
                "图片处理结果".to_string(),   //2
                "图片灰度处理".to_string(),   //3
                "灰度图均值化".to_string(),   //4
                "灰度线性变换".to_string(),   //5
                "快速傅里叶变换".to_string(), //6
                "对比度变换".to_string(),     //7
                "亮度变换".to_string(),       //8
                "高斯模糊".to_string(),       //9
                "图像平滑".to_string(),       //10
                "图像锐化".to_string(),       //11
                "图像分割".to_string(),       //12
            ],
            frame_count: 0,
            image_file_path: String::new(),
            image_data: image::DynamicImage::new(0, 0, image::ColorType::Rgba8),
            state: vec![false, false],
            line_transform: [1, 0],
            adjust_contrast_value: 0.0,
            brightness_value: 0,
            blur_value: 0.0,
            segmentation_index: 0,
        }
    }
}

impl eframe::App for ImageApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.heading(self.string_values[0].clone());

                // 打开图像
                if ui.button(self.string_values[1].clone()).clicked() {
                    let file = FileDialog::new()
                        .add_filter("Image Files", &["png"])
                        .set_directory("/")
                        .pick_file();

                    if let Some(file) = file {
                        self.image_file_path = file.to_str().unwrap().to_string();
                        self.state[0] = true;
                        self.state[1] = true;
                    };
                };

                ui.group(|ui| {
                    ui.set_enabled(self.state[1]);

                    // 图像灰度处理
                    if ui.button(self.string_values[3].clone()).clicked() {
                        let image_data = &self.image_data;
                        let width = image_data.dimensions().0;
                        let height = image_data.dimensions().1;
                        let image_gray = image_to_gray(image_data);

                        let image = image::DynamicImage::ImageLuma8(
                            image::GrayImage::from_vec(width, height, image_gray).unwrap(),
                        );

                        image_output(&image, "cache1.png");
                        self.image_file_path = "./cache1.png".to_string();
                    };

                    // 灰度图均值化
                    if ui.button(self.string_values[4].clone()).clicked() {
                        let image = &self.image_data;
                        let image_gray = image_gray_average(image);
                        let width = image.dimensions().0;
                        let height = image.dimensions().1;

                        let image = image::DynamicImage::ImageLuma8(
                            image::GrayImage::from_vec(width, height, image_gray).unwrap(),
                        );

                        image_output(&image, "cache2.png");
                        self.image_file_path = "./cache2.png".to_string();
                    }

                    // 灰度图线性变换
                    ui.horizontal(|ui| {
                        if ui
                            .add(
                                egui::Slider::new(&mut self.line_transform[1], -255..=255)
                                    .text(self.string_values[5].clone()),
                            )
                            .drag_stopped()
                        {
                            let image = &self.image_data;
                            let width = image.dimensions().0;
                            let height = image.dimensions().1;

                            let a = self.line_transform[0];
                            let b = self.line_transform[1];

                            let image_gray =
                                gray_linear_transfromationg(image, a as f32, b as f32, 255);
                            let image = image::DynamicImage::ImageLuma8(
                                image::GrayImage::from_vec(width, height, image_gray).unwrap(),
                            );

                            image_output(&image, "cache3.png");
                            self.image_file_path = "./cache3.png".to_string();
                        };

                        if ui
                            .add(egui::DragValue::new(&mut self.line_transform[0]).speed(1))
                            .drag_stopped()
                        {
                            let image = &self.image_data;
                            let width = image.dimensions().0;
                            let height = image.dimensions().1;

                            let a = self.line_transform[0];
                            let b = self.line_transform[1];

                            let image_gray =
                                gray_linear_transfromationg(image, a as f32, b as f32, 255);
                            let image = image::DynamicImage::ImageLuma8(
                                image::GrayImage::from_vec(width, height, image_gray).unwrap(),
                            );

                            image_output(&image, "cache3.png");
                            self.image_file_path = "./cache3.png".to_string();
                        }
                    });

                    // 灰度图傅里叶变换
                    if ui.button(self.string_values[6].clone()).clicked() {
                        let image = &self.image_data;
                        let image: GrayImage = gray_fast_fourier_transform(image);

                        image.save("./cache4.png").unwrap();
                        self.image_file_path = "./cache4.png".to_string();
                    }

                    // 图像分割
                    if ui
                        .add(
                            egui::Slider::new(&mut self.segmentation_index, 0..=255)
                                .text(self.string_values[12].clone()),
                        )
                        .drag_stopped()
                    {
                        let image = &self.image_data;
                        let image = segmentate_image(image, self.segmentation_index);

                        image.save("./cache10.png").unwrap();
                    }

                    // 图像对比度变换
                    ui.horizontal(|ui| {
                        if ui
                            .add(egui::DragValue::new(&mut self.adjust_contrast_value).speed(0.1))
                            .drag_stopped()
                        {
                            let image = &self.image_data;
                            let image = adjust_contrast_image(image, self.adjust_contrast_value);

                            image_output(&image, "cache5.png");
                        }

                        ui.monospace(self.string_values[7].clone());
                    });

                    // 图像亮度变换
                    ui.horizontal(|ui| {
                        if ui
                            .add(egui::DragValue::new(&mut self.brightness_value).speed(1))
                            .drag_stopped()
                        {
                            let image = &self.image_data;
                            let image = adjust_brightness_image(image, self.brightness_value);

                            image_output(&image, "cache6.png");
                        }

                        ui.monospace(self.string_values[8].clone());
                    });

                    // 图像高斯模糊
                    ui.horizontal(|ui| {
                        if ui
                            .add(egui::DragValue::new(&mut self.blur_value).speed(0.1))
                            .drag_stopped()
                        {
                            let image = &self.image_data;
                            let image = blur_image(image, self.blur_value);

                            image_output(&image, "cache7.png");
                        }

                        ui.monospace(self.string_values[9].clone());
                    });

                    // 图像平滑
                    if ui.button(self.string_values[10].clone()).clicked() {
                        let image = &self.image_data;
                        let image = smooth_image(image, 1);

                        image_output(&image, "cache8.png");
                    }

                    // 图像锐化
                    if ui.button(self.string_values[11].clone()).clicked() {
                        let image = &self.image_data;
                        let image = sharpen_image(image, 1);

                        image_output(&image, "cache9.png");
                    }

                    // 测试
                    if ui.button("测试").clicked() {
                        println!("hello world");
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |_ui| {
            egui::TopBottomPanel::new(egui::containers::panel::TopBottomSide::Top, "top_panel")
                .show(ctx, |ui| {
                    ui.vertical_centered_justified(|ui| {
                        ui.heading(self.string_values[2].clone());
                    });
                });

            egui::CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if self.image_file_path.is_empty() {
                            ui.label("请先选择图片");
                        } else {
                            ui.image(format!("file://{}", self.image_file_path));

                            if self.state[0] {
                                let image = reading_image(&self.image_file_path);
                                self.image_data = image;
                                self.state[0] = false;
                            };
                        };
                    });
                });
            });
        });

        self.frame_count += 1;
    }
}
