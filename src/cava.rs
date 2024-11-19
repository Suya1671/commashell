use gtk::{
    glib::{self, Object},
    gsk::{Path, PathBuilder},
    prelude::*,
    subclass::prelude::*,
};

glib::wrapper! {
    pub struct Cava(ObjectSubclass<imp::Cava>)
        @extends gtk::Widget;
}

impl Cava {
    pub fn new() -> Self {
        Object::builder().build()
    }
}

mod imp {

    use std::cell::RefCell;

    use astal_cava::prelude::*;
    use glib::Properties;
    use gtk::gsk::{FillRule, PathBuilder};

    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::Cava)]
    pub struct Cava {
        #[property(get, set, nullable)]
        cava: RefCell<Option<astal_cava::Cava>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Cava {
        const NAME: &'static str = "Cava";
        type Type = super::Cava;
        type ParentType = gtk::Widget;
    }

    impl WidgetImpl for Cava {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            // based on https://github.com/kotontrion/kompass/blob/main/kompass/src/cava.vala
            self.parent_snapshot(snapshot);

            let obj = self.obj();
            let Some(cava) = obj.cava() else {
                eprintln!("Cava is not set");
                return;
            };

            let width = obj.width() as f32;
            let height = obj.height() as f32;
            let colour = obj.color();
            let mut values = cava.values();
            let bars = cava.bars();

            let (left_values, right_values) = values.split_at_mut((bars / 2).try_into().unwrap());
            right_values.reverse();

            let left = create_path(
                left_values,
                DrawingDirection::LeftToRight,
                0.0,
                0.0,
                width / 2.0,
                height,
            );

            let right_path = create_path(
                right_values,
                DrawingDirection::RightToLeft,
                width / 2.0,
                0.0,
                width as f32 / 2.0,
                height as f32,
            );

            let final_path = PathBuilder::new();
            final_path.add_path(&left);
            final_path.add_path(&right_path);
            let final_path = final_path.to_path();

            snapshot.append_fill(&final_path, FillRule::Winding, &colour);
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for Cava {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            obj.set_cava(Some(
                astal_cava::functions::default().expect("Expected Cava to exist"),
            ));

            let cava = obj.cava().unwrap();

            cava.set_bars(100);
            cava.set_framerate(90);
            cava.set_noise_reduction(0.22);
            cava.set_stereo(true);

            cava.connect_values_notify(glib::clone!(
                #[weak]
                obj,
                move |_values| {
                    obj.queue_draw();
                }
            ));

            obj.set_css_classes(&["cava"]);
        }
    }
}

fn flip_coord(enabled: bool, screen_dimension: f32, coord: f32) -> f32 {
    let max = 0_f32.max(screen_dimension.min(coord));
    if enabled {
        screen_dimension - max
    } else {
        max
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DrawingDirection {
    LeftToRight = 2,
    RightToLeft = 3,
}

// based on https://github.com/NickvisionApps/Cavalier/blob/main/NickvisionCavalier.Shared/Models/Renderer.cs
fn create_path(
    sample: &[f64],
    direction: DrawingDirection,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> Path {
    let step = height as f32 / (sample.len() as f32 - 1.0);
    let path_builder = PathBuilder::new();
    let flip_image = direction == DrawingDirection::RightToLeft;

    let points: Vec<(f32, f32)> = sample
        .iter()
        .enumerate()
        .map(|(i, value)| (width * *value as f32, step * i as f32))
        .collect();

    let gradients: Vec<f32> = points
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let prev_point = if i == 0 { points[0] } else { points[i - 1] };
            let next_point = if i == points.len() - 1 {
                points[points.len() - 1]
            } else {
                points[i + 1]
            };
            let gradient = prev_point.0 - next_point.0;
            if i == 0 || i == points.len() - 1 {
                gradient
            } else {
                gradient / 2.0
            }
        })
        .collect();

    let x_offset = x;

    path_builder.move_to(
        x_offset + flip_coord(flip_image, width, points[0].0),
        y + points[0].1,
    );

    // take until the second last point
    for (i, point) in points.iter().enumerate().take(points.len() - 1) {
        let gradient = gradients[i];
        let next_point = points[i + 1];
        let next_gradient = gradients[i + 1];

        path_builder.cubic_to(
            x_offset + flip_coord(flip_image, width, point.0 + gradient * 0.5),
            y + point.1 + step * 0.5,
            x_offset + flip_coord(flip_image, width, next_point.0 + next_gradient * -0.5),
            y + next_point.1 + step * -0.5,
            x_offset + flip_coord(flip_image, width, next_point.0),
            y + next_point.1,
        );
    }

    // filling the path
    path_builder.line_to(x_offset + flip_coord(flip_image, width, 0.0), y + height);
    path_builder.line_to(x_offset + flip_coord(flip_image, width, 0.0), y);
    path_builder.close();

    path_builder.to_path()
}
