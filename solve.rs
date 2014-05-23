extern crate collections;
use std::io::File;
use collections::HashMap;
use std::ascii::StrAsciiExt;

#[deriving(Eq, Clone)]
enum TileData<'a> {
  Fixed(char),
  NoWords,
  OneWord(char, &'a str),
  TwoWords(char, &'a str, &'a str)
}

#[deriving(Eq, Clone)]
struct Point {
  x: int,
  y: int
}

#[deriving(Eq, Clone)]
struct Grid<T> {
  width: int,
  height: int,
  tiles: Vec<T>
}

type CrosswordGrid<'a> = Grid<TileData<'a>>;
type StringGrid = Grid<char>;

impl Point {
  #[inline(always)] fn offset(self, x: int, y: int) -> Point {
    Point { x: self.x - x, y: self.y - y }
  }
  #[inline(always)] fn dist(self, other: Point) -> int {
    return std::num::abs(other.x - self.x) + std::num::abs(other.y - self.y);
  }
}

impl<T> Grid<T> {
  #[inline(always)] fn is_valid(& self, p: Point) -> bool {
    if p.x < 0 || p.x >= self.width { false }
    else if p.y < 0 || p.y >= self.height { false }
    else { true }
  }
  #[allow(dead_code)]
  #[inline(always)] fn get_point(& self, p: Point) -> Option<Point> {
    if self.is_valid(p) { Some(p) }
    else                { None    }
  }
  #[inline(always)] fn set(& mut self, p: Point, data: T) {
    *self.tiles.get_mut((self.height * p.y + p.x) as uint) = data;
  }
  #[inline(always)] fn get_ref<'a>(&'a self, p: Point) -> Option<&'a T> {
    if self.is_valid(p) { Some(self.tiles.get((self.height * p.y + p.x) as uint)) }
    else { None }
  }
  #[allow(dead_code)]
  #[inline] fn get_mut_ref<'a>(&'a mut self, p: Point) -> Option<&'a mut T> {
    if self.is_valid(p) { Some(self.tiles.get_mut((self.height * p.y + p.x) as uint)) }
    else { None }
  }
  fn map<U>(& self, map: |&T|->U)->Grid<U> {
    let mapped: Vec<U> = self.tiles.iter().map(map).collect();
    Grid { width: self.width, height: self.height, tiles: mapped }
  }
}
impl<T: Clone> Grid<T> {
  #[inline(always)] fn replace(& self, p: Point, data: T) -> Grid<T> {
    assert!(self.is_valid(p));
    let mut new = self.clone();
    new.set(p, data);
    new
  }
}

impl std::fmt::Show for StringGrid {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
    let iter = self.tiles.as_slice().chunks(self.width as uint);
    let pieces: Vec<~str> = iter.map(|x| std::str::from_chars(x)).collect();
    let string = pieces.connect("\n");
    write!(fmt, "{}", string)
  }
}

impl<'a> Grid<TileData<'a>> {
  fn neighbors<'b>(& self, p: Point, invec: &'b mut [Point, ..4]) -> &'b [Point] {
    let offsets = [
      p.offset(-1, 0),
      p.offset( 1, 0),
      p.offset( 0,-1),
      p.offset( 0, 1)
    ];
    let mut i = 0;
    for &offset in offsets.iter() {
      if self.is_valid(offset) {
        invec[i] = offset;
        i += 1;
      }
    }
    invec.slice_to(i)
  }

  #[allow(dead_code)]
  fn to_strgrid(&self) -> StringGrid {
    self.map(default_char)
  }
}

trait FlattenCrossword {
  fn flatten(&self) -> StringGrid;
  fn flattenWord(&self, &str) -> StringGrid;
}

impl<'a> FlattenCrossword for Vec<CrosswordGrid<'a>> {
  fn flatten(&self) -> StringGrid {
    let mut iter = self.iter().map(|x| x.tiles.iter().map(default_char).collect::<Vec<char>>());
    let next = iter.next().unwrap();
    let folded = iter.fold(next, |accum, x| {
      accum.iter().zip(x.iter()).map(|(&a, &b)| {
        if a == b {  a  }
        else        { ' ' }
      }).collect::<Vec<char>>()
    });
    Grid { width: self.get(0).width, height: self.get(0).height, tiles: folded }
  }
  fn flattenWord(&self, s: &str) -> StringGrid {
    let mapped: Vec<CrosswordGrid<'a>> = self.iter().map(|x| x.map(|&tile| match tile {
      OneWord(_, word) if word == s => tile,
      TwoWords(_, a, b) if a == s || b == s => tile,
      Fixed(_) => tile,
      _ => NoWords
    })).collect();
    mapped.flatten()
  }
}

trait Case {
  fn to_upper(&self) -> char;
  fn to_lower(&self) -> char;
}
impl Case for char {
  #[inline(always)] fn to_upper(&self) -> char {
    self.to_ascii().to_upper().to_char()
  }

  #[inline(always)] fn to_lower(&self) -> char {
    self.to_ascii().to_lower().to_char()
  }
}

#[inline] fn default_char<'a>(tile: &TileData<'a>) -> char {
  match *tile {
    Fixed(c) => c.to_upper(),
    OneWord(c, _) => c.to_lower(),
    TwoWords(c, _, _) => c.to_lower(),
    NoWords => ' '
  }
}

fn readlines(file: &str) -> Vec<~str> {
  let path = Path::new(file);
  let input = File::open(&path).read_to_end().unwrap();
  let text = std::str::from_utf8(input.as_slice()).unwrap();
  text.lines_any().map(|line| line.into_owned()).collect()
}

fn readgrid(file: &str) -> Box<CrosswordGrid> {
  let lines = readlines(file);
  let longest = lines.iter().map(|a| a.char_len()).max().unwrap();
  let mut full = StrBuf::new();
  for line in lines.iter() {
    full.push_str(*line);
  }
  let downcase = full.into_owned().to_ascii_lower();
  let mut tileit = downcase
  .chars()
  .map(|c| match c {
    ' ' => NoWords,
    _   => Fixed(c)
  });
  let tiles: Vec<TileData> = tileit.collect();
  box Grid { width: longest as int, height: lines.len() as int, tiles: tiles }
}

fn readwords(file: &str) -> Vec<~str> {
  let lines = readlines(file);
  lines.iter().map(|x| x.to_ascii_lower()).collect::<Vec<~str>>()
}

fn hashgrid(grid: CrosswordGrid) -> HashMap<char, Point> {
  let mut map = HashMap::<char, Point>::new();
  for x in range(0, grid.width) {
    for y in range(0, grid.height) {
      let p = Point { x: x, y: y };
      let data = *grid.get_ref(p).unwrap();
      match data {
        Fixed(letter) => {
          match map.find(&letter) {
            Some(x) => {
              fail!("Already have letter: \"{}\" at point: {},{}", letter, x.x, x.y);
            },
            None => { }
          }
          map.insert(letter, p);
        },
        _ => { }
      }
    }
  }
  map
}

fn allpaths<'a>(grid: & CrosswordGrid<'a>, word: &'a str, start: Point, dest: Point, s: &'a str, accum: &mut Vec<CrosswordGrid<'a>>) {
  let len = s.len() as int - 1;
  if start == dest && len == 0 {
    accum.push(grid.clone());
    return;
  }
  if start.dist(dest) > len {
    return;
  }
  match grid.get_ref(start) {
    None => { },
    // skip if wrong character, or the character is part of this word
    Some(&OneWord(c, w)) if c == s.char_at(0) && w != word => {
      let newtile: TileData<'a> = TwoWords(c, w, word);
      let newgrid = grid.replace(start, newtile);
      allpaths2(&newgrid, word, start, dest, s, accum)
    }
    Some(&NoWords) => {
      let newtile = OneWord(s.char_at(0), word);
      let newgrid = grid.replace(start, newtile);
      allpaths2(&newgrid, word, start, dest, s, accum)
    }
    Some(&TwoWords(..)) => { },
    _ => { }
  }
}

fn allpaths2<'a>(grid: & CrosswordGrid<'a>, word: &'a str, start: Point, dest: Point, s: &'a str, accum: &mut Vec<CrosswordGrid<'a>>) {
  let mystring: & str = s.slice_from(1);

  let mut tmpvec = [Point { x: 0, y: 0}, ..4];
  let neighbors = grid.neighbors(start, &mut tmpvec);
  for &p in neighbors.iter() {
      allpaths(grid, word, p, dest, mystring, accum);
  }
}

fn word_to_path(gridmap: &HashMap<char, Point>, word: &str) -> (Point, Point) {
  let first = word.char_at(0);
  let last = word.char_at_reverse(word.len()); // no, really!
  let start = gridmap.get(&first);
  let end = gridmap.get(&last);
  (*start, *end)
}

fn add_word<'a>(accum: Vec<CrosswordGrid<'a>>, wordpt: &[(&'a str, &(Point, Point))])-> Vec<CrosswordGrid<'a>> {
  let next = wordpt.head();
  if next.is_none() {
    accum
  } else {
    let (word, &(start, end)) = *next.unwrap();
    println!("searching \"{}\" on {} grids", word, accum.len());
    let mut out: Vec<CrosswordGrid> = Vec::new();
    for i in accum.iter() {
      allpaths2(i, word, start, end, word, &mut out);
    }
    if out.len() > 0 {
      add_word(out, wordpt.slice_from(1))
    } else {
      println!("could not produce any paths to fit \"{}\"!", word);
      accum
    }
  }
}

fn main() {
  let args = std::os::args();
  let blankgrid = *readgrid(args.get(1).as_slice());
  let mut words = readwords(args.get(2).as_slice());
  words.sort_by(|a,b| a.len().cmp(&b.len()));
  let gridmap: HashMap<char, Point> = hashgrid(blankgrid.clone());
  let paths: Vec<(Point, Point)> = words.iter().map(|word| word_to_path(&gridmap, *word)).collect();
  println!("loaded {} words!", words.len());
  for _ in range(0, std::int::parse_bytes(args.get(3).as_bytes(), 10).unwrap()) {
    let worditer = words.iter().map(|x| x.as_slice());
    let wordpts: Vec<(&str, &(Point, Point))> = worditer.zip(paths.iter()).collect();
    let results = add_word(vec!(blankgrid.clone()), wordpts.as_slice());
    println!("{}", results.flatten());
    for w in words.iter() {
      println!("Showing only \"{}\":", *w);
      println!("{}", results.flattenWord(*w));
    }
  }
}
