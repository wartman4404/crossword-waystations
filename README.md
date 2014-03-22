This is a solver for the Wall Street Journal's [Waystations](http://blogs.wsj.com/puzzle/2014/02/14/way-stations-saturday-puzzle/tab/puzzles/) puzzle, appearing (so far) only on February 14, 2014.

The same implementation is written in both Rust and Scala, to help me learn the former, and to highlight some commonalities for anyone else coming from one to the other.

Suggestions to improve either version are very welcome.

---

The rules to Waystations are pretty straightforward:
- Each word follows a continuous path through the grid, beginning and ending at one of the pre-filled letters.  It cannot intersect pre-filled letters at other points, or cross over itself.
- Otherwise, everything is allowed.  Words can intersect letters belonging to other words.  Unlike the words in a regular crossword puzzle, which can only go up and down or side-to-side, these words can make turns and go slantways, longways, backways, squareways, frontways, and any other ways you can think of.
- When the puzzle is completed, each tile must be used by exactly two words.

---

#### Usage:

Rust 0.10-pre:

    rustc solve.rs
    ./solve.rs grid words

Scala 2.10:

    scalac solve.scala
    scala Main grid words
    
---

#### Areas for improvement

At the time of writing, Rust is still under development, and its syntax and standard library continue to evolve.  It's very possible that you'll need to make some alterations in order for solve.rs to build without warnings.

The breadth-first recursive search employed by the solver could be better, as could the ordering of words to search.

When compiling with rustc --opt-level=3 and scalac -optimise, the scala implementation is about 10% faster.  This is a bit of a surprise.
