pub struct TerrainGrid {
    pub width: usize,
    pub height: usize,
    pub heights: Vec<u8>,
    pub sand_weights: Vec<f32>,
    pub is_passable: Vec<bool>,
}

impl TerrainGrid {
    pub fn new(width: usize, height: usize) -> Self {
        let mut grid = Self {
            width,
            height,
            heights: vec![0; width * height],
            sand_weights: vec![0.0; (width + 1) * (height + 1)],
            is_passable: vec![true; width * height],
        };

        for y in 0..height {
            for x in 0..width {
                let h = ((x as f32 * 0.15).sin() * (y as f32 * 0.15).cos() * 6.0).abs() as u8;
                grid.set_height(x, y, h);
                grid.set_is_passable(x, y, true);
            }
        }
        grid
    }

    fn index(&self, x: usize, y: usize) -> Option<usize> {
        if x < self.width && y < self.height {
            Some(y * self.width + x)
        } else {
            None
        }
    }

    pub fn get_height(&self, x: usize, y: usize) -> Option<u8> {
        self.index(x, y).map(|index| self.heights[index])
    }

    pub fn set_height(&mut self, x: usize, y: usize, value: u8) {
        if let Some(index) = self.index(x, y) {
            self.heights[index] = value;
        }
    }

    fn vertex_index(&self, vx: usize, vy: usize) -> Option<usize> {
        if vx <= self.width && vy <= self.height {
            Some(vy * (self.width + 1) + vx)
        } else {
            None
        }
    }

    pub fn add_sand_weight_in_radius(&mut self, cx: usize, cy: usize, radius: isize, amount: f32) {
        if radius <= 0 {
            return;
        }

        let radius_f = radius as f32;
        let center_x = cx as isize;
        let center_y = cy as isize;

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let nx = center_x + dx;
                let ny = center_y + dy;

                if nx < 0 || ny < 0 || nx > self.width as isize || ny > self.height as isize {
                    continue;
                }

                let distance = ((dx * dx + dy * dy) as f32).sqrt();
                if distance > radius_f {
                    continue;
                }

                if let Some(index) = self.vertex_index(nx as usize, ny as usize) {
                    let falloff = 1.0 - (distance / radius_f);
                    let weight_to_add = amount * falloff;
                    self.sand_weights[index] = (self.sand_weights[index] + weight_to_add).clamp(0.0, 1.0);
                }
            }
        }
    }

    pub fn get_sand_weight(&self, vx: usize, vy: usize) -> f32 {
        self.vertex_index(vx, vy).map(|index| self.sand_weights[index]).unwrap_or(0.0)
    }

    pub fn get_is_passable(&self, x: usize, y: usize) -> Option<bool> {
        self.index(x, y).map(|index| self.is_passable[index])
    }

    pub fn set_is_passable(&mut self, x: usize, y: usize, value: bool) {
        if let Some(index) = self.index(x, y) {
            self.is_passable[index] = value;
        }
    }

    pub fn clear_sand_weights(&mut self) {
        for v in self.sand_weights.iter_mut() {
            *v = 0.0;
        }
    }
}