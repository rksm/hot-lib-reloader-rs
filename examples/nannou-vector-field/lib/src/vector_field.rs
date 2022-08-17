use nannou::{color::ConvertInto, prelude::*};

#[derive(Debug)]
pub(crate) struct VectorField<const ROWS: usize, const COLS: usize> {
    vectors: [[(Vec2, Vec2); COLS]; ROWS],
    bounds: Rect,
}

impl<const ROWS: usize, const COLS: usize> Default for VectorField<ROWS, COLS> {
    fn default() -> Self {
        Self {
            vectors: [[(Vec2::ZERO, Vec2::ZERO); COLS]; ROWS],
            bounds: Rect::from_w_h(0.0, 0.0),
        }
    }
}

impl<const ROWS: usize, const COLS: usize> VectorField<ROWS, COLS> {
    pub fn new(bounds: Rect, f: impl Fn(Vec2) -> Vec2) -> Self {
        let mut result = Self::default();

        (0..ROWS).for_each(|row| {
            let y = row as f32 / ROWS as f32 * bounds.h() + bounds.bottom();
            (0..COLS).for_each(|col| {
                let x = col as f32 / COLS as f32 * bounds.w() + bounds.left();
                let pos = pt2(x, y);
                let vec = f(pos).normalize_or_zero();
                result.vectors[row][col] = (pos, vec);
            });
        });

        result.bounds = bounds;
        result
    }

    pub(crate) fn draw(&self, draw: &Draw) {
        for row in &self.vectors {
            for (pos, vec) in row {
                draw.arrow()
                    .start(*pos)
                    .end(*pos + (*vec * 20.0))
                    .stroke_weight(2.0);
            }
        }
    }

    pub fn get_vector(&self, pos: Vec2) -> Vec2 {
        let col = ((pos.x - self.bounds.left()) / self.bounds.w() * COLS as f32)
            .max(0.0)
            .min((COLS - 1) as f32)
            .round() as usize;
        let row = ((pos.y - self.bounds.bottom()) / self.bounds.h() * ROWS as f32)
            .max(0.0)
            .min((ROWS - 1) as f32)
            .round() as usize;

        self.vectors
            .get(row)
            .and_then(|rows| rows.get(col))
            .map(|(_, vec)| *vec)
            .unwrap_or_default()
    }

    pub fn update(&mut self, f: impl Fn(Vec2, Vec2) -> Vec2) {
        let bounds = self.bounds;
        (0..ROWS).for_each(|row| {
            (0..COLS).for_each(|col| {
                let (pos, vec) = self.vectors[row][col];
                self.vectors[row][col] = (pos, f(pos, vec))
            });
        });
    }
}
