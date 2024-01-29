#[derive(Default)]
pub struct Pattern {
    rows_data: Vec::<Row>,
    rows_count: usize,
}

impl Pattern {

    pub fn add_row(&mut self, cols: usize) {
        let mut cell_data = vec![];

        for _ in 0..cols {
            cell_data.push(Cell::Base);
        }

        self.rows_data.push(Row {
            cell_data,
            left_offset: 0,
            cell_count : cols
        });

        self.rows_count += 1;
    }

    pub fn shift_right(&mut self, row: usize) {
        self.rows_data[row].cell_data.insert(0, Cell::Base);
        self.rows_data[row].left_offset += 1;
        self.rows_data[row].cell_count += 1;
    }

    pub fn shift_left(&mut self, row: usize) {
        if self.rows_data[row].left_offset > 0 {
            self.rows_data[row].left_offset -= 1;
        } else {
            self.rows_data[row].cell_data.remove(0);
        }

        self.rows_data[row].cell_count -= 1;
    }


    pub fn add_col_left(&mut self, row: usize) {
        // simple is just insert data at start
        self.rows_data[row].cell_data.insert(0, Cell::Base);
        self.rows_data[row].cell_count += 1;

    }


    pub fn remove_col_left(&mut self, row: usize) {
        self.rows_data[row].left_offset += 1;
    }

    pub fn add_col_right(&mut self, row: usize) {
        if self.rows_data[row].cell_data.len() == self.rows_data[row].cell_count {
            self.rows_data[row].cell_data.push(Cell::Base);

        }

        self.rows_data[row].cell_count += 1;
    }

    pub fn remove_col_right(&mut self, row: usize) {
        self.rows_data[row].cell_count -= 1;
    }

    pub fn cell(&self, row: usize, col: usize) -> &Cell {
        &self.rows_data[row].cell_data[col]
    }

    pub fn cell_mut(&mut self, row: usize, col: usize) -> &mut Cell {
        &mut self.rows_data[row].cell_data[col]
    }

    pub fn rows(&self) -> usize {
        self.rows_count
    }

    pub fn cols(&self, row: usize) -> usize {
        self.rows_data[row].cell_count
    }

    pub fn left_start(&self, row: usize) -> usize {
        self.rows_data[row].left_offset
    }

}


struct Row {
    cell_data : Vec::<Cell>,
    left_offset: usize,
    cell_count: usize
}

#[derive(Copy, Clone, Debug)]
pub enum Cell {
    Base,
    Color1,
    // tag ud,
    // tag ind
    // andre ting som man kan tænke sig
}

impl Cell {
    pub fn is_base(&self) -> bool {
        match self {
            Self::Base => true,
            _ => false
        }
    }

    pub fn is_color(&self) -> bool {
        match self {
            Self::Color1 => true,
            _ => false
        }
    }
}
