import scala.collection.mutable.Buffer



object Main {
  sealed abstract class TileData()
  final case class Fixed(c: Char) extends TileData()
  final case class OneWord(c: Char, wordOne: String) extends TileData()
  final case class TwoWords(c: Char, wordOne: String, wordTwo: String) extends TileData()
  final case object NoWords extends TileData()

  case class Point(x: Int, y: Int) {
    def offset(x: Int, y: Int) = Point(this.x + x, this.y + y);
    def dist(other: Point) = Math.abs(this.x - other.x) + Math.abs(this.y - other.y);
  }

  case class Grid[T](width: Int, height: Int, tiles: Array[T]) {
    def is_valid(p: Point) = {
      if (p.x < 0 || p.x >= this.width) { false }
      else if (p.y < 0 || p.y >= this.height) { false }
      else { true }
    }
    def get_point(p: Point) = {
      if (is_valid(p)) { Some(p) }
      else             { None }
    }
    def set(p: Point, data: T) = this.tiles(p.y * this.width + p.x) = data
    def get(p: Point) = get_point(p).map(p => this.tiles( p.y * this.width + p.x ))
    def map[U](map: (T)=>U)(implicit classtag: scala.reflect.ClassTag[U]) = {
      this.copy(tiles = tiles.map(map).toArray)
    }
    def replace(p: Point, data: T)(implicit classtag: scala.reflect.ClassTag[T]) = {
      if (!is_valid(p)) {
        throw new IllegalArgumentException("cannot write %s to illegal point %s".format(data, p))
      }
      val newarray = new Array[T](this.tiles.length)
      System.arraycopy(this.tiles, 0, newarray, 0, this.tiles.length)
      val newgrid = this.copy(tiles = newarray)
      newgrid.set(p, data)
      newgrid
    }
  }

  implicit class CrosswordGridExt(grid: Grid[TileData]) {
    def neighbors(p: Point) = {
      Array(
        p.offset(-1, 0),
        p.offset( 1, 0),
        p.offset( 0,-1),
        p.offset( 0, 1))
      .flatMap(grid.get_point)
    }
  }

  trait StringGridTrait extends Grid[Char] {
    override def toString() = {
      val pieces = tiles.grouped(width).map(_.mkString)
      pieces.mkString("\n")
    }
  }

  type CrosswordGrid = Grid[TileData]
  type StringGrid = Grid[Char]

  implicit class FlattenCrossword(grids: Seq[CrosswordGrid]) {
    def flattenAsCrossword(): StringGrid = {
      val tiles = grids.map(_.tiles.map(defaultChar))
      val folded = tiles.reduceLeft((accum, x) => {
        accum.zip(x).map({ case(a, b) => {
          if (a == b)  a
          else        ' '
        }})
      })
      new StringGrid(grids(0).width, grids(0).height, folded) with StringGridTrait
    }
    def flattenWord(s: String): StringGrid = {
      val mapped = grids.map(_.map(tile => tile match {
        case OneWord(_, word) if word == s => tile
        case TwoWords(_, a, b) if a == s || b == s => tile
        case Fixed(_) => tile
        case _ => NoWords
      }))
      mapped.flattenAsCrossword()
    }
  }

  def defaultChar(tile: TileData) = tile match {
    case Fixed(c) => c.toUpper
    case OneWord(c, _) => c.toLower
    case TwoWords(c, _, _) => c.toLower
    case _  => ' '
  }

  def readlines(file: String) = {
    val source = scala.io.Source.fromFile(file)
    val ret = source.getLines.toArray;
    source.close()
    ret
  }

  def readgrid(file: String) = {
    val lines = readlines(file)
    val longest = lines.map(_.length).max
    val full = new StringBuilder()
    for (line <- lines) {
      full.append(line)
    }
    val upcase = full.toString().toUpperCase()
    val tiles = upcase.map(c => c match {
        case ' ' => NoWords
        case any => Fixed(any)
      })
    new CrosswordGrid(longest, lines.length, tiles.toArray)
  }

  def readwords(file: String) = {
    val lines = readlines(file)
    lines
  }

  def hashgrid(grid: CrosswordGrid) = {
    var map: Map[Char, Point] = Map.empty
    for (x <- 0 until grid.width;
         y <- 0 until grid.height) {
      val p = Point(x,y)
      val data = grid.get(p).get;
      data match {
        case Fixed(letter) => {
          map.get(letter) match {
            case Some(x) => {
              println("Already have letter: %s at point %s".format(letter, x))
              System.exit(-1)
            }
            case None => { }
          }
          map = map + (letter -> p)
        }
        case _ => { }
      }
    }
    map
  }
  
  def allpaths(grid: CrosswordGrid, word: String, start: Point, dest: Point, s: String, accum: Buffer[CrosswordGrid]): Unit = {
    val len = s.length - 1
    if (start == dest && len == 0) {
      accum += grid
      return;
    }
    if (start.dist(dest) > len) {
      return;
    }
    grid.get(start) match {
      case None => Array()
      // skip if wrong character, or the character is part of this word
      case Some(OneWord(c, w)) if c == s(0) && w != word => {
        val newtile = TwoWords(c, w, word)
        val newgrid = grid.replace(start, newtile)
        allpaths2(newgrid, word, start, dest, s, accum)
      }
      case Some(NoWords) => {
        val newtile = OneWord(s(0), word)
        val newgrid = grid.replace(start, newtile)
        allpaths2(newgrid, word, start, dest, s, accum)
      }
      case Some(TwoWords(_, _, _)) => Array()
      case _  => Array()
    }
  }

  def allpaths2(grid: CrosswordGrid, word: String, start: Point, dest: Point, s: String, accum: Buffer[CrosswordGrid]): Unit = {
    val mystring = s.substring(1)

    val neighbors = grid.neighbors(start)
    for (p <- neighbors) {
      allpaths(grid, word, p, dest, mystring, accum)
    }
  }

  def word_to_path(gridmap: Map[Char, Point], word: String) = {
    val upper = word.toUpperCase()
    val (first, last) = (upper.head, upper.last)
    val start = gridmap.get(first).get
    val end = gridmap.get(last).get
    (start, end)
  }

  def add_word(accum: Buffer[CrosswordGrid], wordpt: Array[(String, (Point, Point))]): Buffer[CrosswordGrid] = {
    val next = wordpt.headOption
    if (next.isEmpty) {
      accum
    } else {
      val (word, (start, end)) = next.get
      println("""searching "%s" on %d grids""".format(word, accum.length))
      val out = Buffer[CrosswordGrid]()
      for (i <- accum) {
        allpaths2(i, word, start, end, word, out)
      }
      if (out.length > 0) {
        add_word(out, wordpt.tail)
      } else {
        println("""could not produce any paths to fit "%s"!""".format(word))
        accum
      }
    }
  }

  def main(args: Array[String]): Unit = {
    val blankgrid = readgrid(args(0))
    val unsortedWords = readwords(args(1))
    val words = unsortedWords.sortBy(word => word.length)
    val gridmap = hashgrid(blankgrid)
    val paths: Array[(Point, Point)] = words.map(word => word_to_path(gridmap, word))
    println("loaded %d words!".format(words.length))
    val wordpts = words.zip(paths)
    val results = add_word(Buffer(blankgrid), wordpts)
    println("%s".format(results.flattenAsCrossword()))

    for (w <- words) {
      println("""Showing only "%s":""".format(w))
      println("%s".format(results.flattenWord(w)))
    }
  }
}
