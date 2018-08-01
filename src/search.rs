#![allow(dead_code)]

use grep2::matcher::Matcher;
use grep2::printer::{JSON, JSONBuilder, Standard, StandardBuilder, Stats};
use grep2::searcher::Searcher;
use encoding_rs::Encoding;
use termcolor::WriteColor;

/// The configuration for the search worker. Among a few other things, the
/// configuration primarily controls the way we show search results to users
/// at a very high level.
#[derive(Clone, Debug)]
struct Config {
    encoding: Option<&'static Encoding>,
    output: Output,
    stats: bool,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            encoding: None,
            output: Output::default(),
            stats: false,
        }
    }
}

/// The output mode for a search worker.
#[derive(Clone, Debug)]
pub enum Output {
    /// Use the standard printer, which supports the classic grep-like format.
    Standard {
        /// A configured builder for constructing the standard printer.
        builder: StandardBuilder,
        /// The format emitted by the printer.
        kind: OutputKind,
    },
    /// A JSON printer, which emits results in the JSON Lines format.
    ///
    /// This only supports one output mode.
    JSON {
        /// A configured builder for constructing the JSON printer.
        builder: JSONBuilder,
    },
}

/// The output mode for the standard printer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputKind {
    /// The classic grep-like format.
    Classic,
    /// Show only a count of the total number of matches (counting each line
    /// at most once) found.
    ///
    /// If the `path` setting is enabled, then the count is prefixed by the
    /// corresponding file path.
    Count,
    /// Show only a count of the total number of matches (counting possibly
    /// many matches on each line) found.
    ///
    /// If the `path` setting is enabled, then the count is prefixed by the
    /// corresponding file path.
    CountMatches,
    /// Show only the file path if and only if a match was found.
    ///
    /// This ignores the `path` setting and always shows the file path.
    FilesWithMatches,
    /// Show only the file path if and only if a match was found.
    ///
    /// This ignores the `path` setting and always shows the file path.
    FilesWithoutMatch,
    /// Don't show any output and the stop the search once a match is found.
    Quiet,
}

impl Default for Output {
    fn default() -> Output {
        Output::Standard {
            builder: StandardBuilder::new(),
            kind: OutputKind::default(),
        }
    }
}

impl Default for OutputKind {
    fn default() -> OutputKind {
        OutputKind::Classic
    }
}

/// A builder for configuring and constructing a search worker.
#[derive(Clone, Debug)]
pub struct SearchWorkerBuilder {
    config: Config,
}

impl Default for SearchWorkerBuilder {
    fn default() -> SearchWorkerBuilder {
        SearchWorkerBuilder::new()
    }
}

impl SearchWorkerBuilder {
    /// Create a new builder for configuring and constructing a search worker.
    pub fn new() -> SearchWorkerBuilder {
        SearchWorkerBuilder { config: Config::default() }
    }

    pub fn build<M, W>(
        &self,
        searcher: Searcher,
        matcher: M,
        wtr: W,
    ) -> SearchWorker<M, W>
    where M: Matcher,
          W: WriteColor
    {
        let config = self.config.clone();
        let stats = if config.stats { Some(Stats::new()) } else { None };
        let wtr = Writer::new(&self.config.output, wtr);
        SearchWorker { config, searcher, matcher, wtr, stats }
    }

    /// The type of encoding to use to read the source data. When this is set,
    /// the source data is transcoded from the specified encoding to UTF-8
    /// before being searched.
    pub fn encoding(
        &mut self,
        encoding: Option<&'static Encoding>,
    ) -> &mut SearchWorkerBuilder {
        self.config.encoding = encoding;
        self
    }

    /// Set the output mode for this searcher.
    pub fn output(
        &mut self,
        output: Output,
    ) -> &mut SearchWorkerBuilder {
        self.config.output = output;
        self
    }

    /// Compute statistics when enabled. This includes, but is not limited to,
    /// the total number of matches, the number of bytes searched and more.
    ///
    /// When statistics are computed, they are included in the results of every
    /// search.
    pub fn stats(&mut self, yes: bool) -> &mut SearchWorkerBuilder {
        self.config.stats = yes;
        self
    }
}

#[derive(Debug)]
pub struct SearchWorker<M, W> {
    config: Config,
    searcher: Searcher,
    matcher: M,
    wtr: Writer<W>,
    stats: Option<Stats>,
}

impl<M: Matcher, W: WriteColor> SearchWorker<M, W> {
}

/// The writer for a search worker.
///
/// The `W` type parameter refers to the type of the underlying writer.
#[derive(Debug)]
enum Writer<W> {
    /// Use the standard printer, which supports the classic grep-like format.
    Standard {
        /// A printer, which can cheaply build implementations of Sink.
        printer: Standard<W>,
        /// The format emitted by the printer.
        kind: OutputKind,
    },
    /// A JSON printer, which emits results in the JSON Lines format.
    ///
    /// This only supports one output mode.
    JSON {
        /// A printer, which can cheaply build implementations of Sink.
        printer: JSON<W>,
    },
}

impl<W: WriteColor> Writer<W> {
    fn new(output: &Output, wtr: W) -> Writer<W> {
        match *output {
            Output::Standard { ref builder, kind } => {
                Writer::Standard {
                    printer: builder.build(wtr),
                    kind: kind,
                }
            }
            Output::JSON { ref builder } => {
                Writer::JSON {
                    printer: builder.build(wtr),
                }
            }
        }
    }
}
