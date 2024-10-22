use glam::Vec3;

use crate::{graph::Graph, motif_assembler::Motif};

impl Graph {
    pub fn to_schematic(&self, path: &str) {
        log::info!("Plotting schematic.");

        use plotters::{
            chart::{ChartBuilder, ChartContext}, 
            prelude::{Circle, DrawingArea, IntoDrawingArea, SVGBackend}, 
            series::LineSeries, 
            style::{ShapeStyle, BLACK, WHITE}
        };

        let motifs: Vec<Motif> = self.to_motifs().collect();
        let edges = self.iter_edges();

        let mut x_min: f32 = self.iter_vertices().min_by(|a, b| a.position.x.partial_cmp(&b.position.x).unwrap()).unwrap().position.x;
        let mut x_max: f32 = self.iter_vertices().max_by(|a, b| a.position.x.partial_cmp(&b.position.x).unwrap()).unwrap().position.x;
        let mut y_min: f32 = self.iter_vertices().min_by(|a, b| a.position.x.partial_cmp(&b.position.y).unwrap()).unwrap().position.y;
        let mut y_max: f32 = self.iter_vertices().max_by(|a, b| a.position.x.partial_cmp(&b.position.y).unwrap()).unwrap().position.y;

        x_min -= 0.05 * (x_max - x_min);
        x_max += 0.05 * (x_max - x_min);
        y_min -= 0.05 * (y_max - y_min);
        y_max += 0.05 * (y_max - y_min);

        // Plot
        {
            log::trace!("Creating chart");
            let canvas: DrawingArea<_,_> = SVGBackend::new(
                path,
                (800,800)
            ).into_drawing_area();

            canvas.fill(&WHITE).unwrap();

            let mut chart: ChartContext<'_,_,_> = ChartBuilder::on(&canvas)
                .build_cartesian_2d(x_min..x_max, y_min..y_max)
                .unwrap();

            // Draw edges
            log::trace!("Drawing edges");
            edges.for_each(|edge| {
                chart.draw_series(
                    LineSeries::new(
                        (0..2).map(|i| if i == 0 {
                            let pos: Vec3 = self.vertices.get(edge.origin).unwrap().position;
                            (pos.x, pos.y)
                        } else {
                            let pos: Vec3 = self.vertices.get(edge.destination).unwrap().position;
                            (pos.x, pos.y)
                        }),
                        &BLACK
                    )
                ).unwrap();
            });

            // Draw motifs
            // Hairpins
            log::trace!("Drawing hairpins");
            chart.draw_series(
                motifs.iter().enumerate().filter(|&(_index, motif)| match motif {
                    Motif::Hairpin(_) => true,
                    _ => false
                }).map(|(index, _motif)| {
                    let pos: Vec3 = self.vertices.get(index).unwrap().position;
                    Circle::new((pos.x, pos.y), 5, ShapeStyle::from(&BLACK).filled())
                })
            ).unwrap();
            // Kinks
            log::trace!("Drawing kinks");
            chart.draw_series(
                motifs.iter().enumerate().filter(|&(_index, motif)| *motif == Motif::Kink90).map(|(index, _motif)| {
                    let pos: Vec3 = self.vertices.get(index).unwrap().position;
                    Circle::new((pos.x, pos.y), 5, ShapeStyle::from(&BLACK))
                })
            ).unwrap();
            // Junctions
            log::trace!("Drawing junctions");
            (0..2).for_each(|i| {
                chart.draw_series(
                    motifs.iter().enumerate().filter(|&(_index, motif)| *motif == Motif::O3WJ || *motif == Motif::O4WJ).map(|(index, _motif)| {
                        let pos: Vec3 = self.vertices.get(index).unwrap().position;
                        if i == 1 {
                            Circle::new((pos.x, pos.y), 5, ShapeStyle::from(&BLACK).filled())
                        } else {
                            Circle::new((pos.x, pos.y), 5, ShapeStyle::from(&BLACK))
                        }
                    })
                ).unwrap();
            });
            // Kissing Loop
            log::trace!("Drawing kissing loops");
            (0..2).for_each(|i| {
                chart.draw_series(
                    motifs.iter().enumerate().filter(|&(_index, motif)| match motif {
                        Motif::KL(_) => true,
                        _ => false
                    }).map(|(index, _motif)| {
                        let pos: Vec3 = self.vertices.get(index).unwrap().position;
                        if i == 1 {
                            Circle::new((pos.x, pos.y), 5, ShapeStyle::from(&WHITE).filled())
                        } else {
                            Circle::new((pos.x, pos.y), 7, ShapeStyle::from(&BLACK).filled())
                        }
                    })
                ).unwrap();
            });
        }
        
    }

}