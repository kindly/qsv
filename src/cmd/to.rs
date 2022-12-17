static USAGE: &str = r#"
Convert CSV files to XLSX/POSTGRES/SQLITE/PARQUET

POSTGRES
To convert to postgres you need to supply connection string. The format is decribed https://docs.rs/postgres/latest/postgres/config/struct.Config.html#examples-1.
Additionaly you can use `env=MY_ENV_VAR` and that will get the connection string from the enviroment variable `MY_ENV_VAR`.

Examples:

Load `file1.csv` and `file2.csv' file to local database `test`, with user `testuser`, and password `pass`.

  $ qsv to postgres 'postgres://testuser:pass@localhost/test' file1.csv file2.csv

Load same files into a new/existing postgres schema `myschema`

  $ qsv to postgres 'postgres://testuser:pass@localhost/test' --schema=myschema file1.csv file2.csv

Load same files into a new/existing postgres database whose connection string is in the `DATABASE_URL` environment variable.

  $ qsv to postgres 'env=DATABASE_URL' file1.csv file2.csv

Drop tables if they exist before loading.

  $ qsv to postgres 'postgres://testuser:pass@localhost/test' --drop file1.csv file2.csv

Evolve tables if they exist before loading. Read http://datapackage_convert.opendata.coop/evolve.html to explain how evolving works.

  $ qsv to postgres 'postgres://testuser:pass@localhost/test' --evolve file1.csv file2.csv


SQLITE
Convert to sqlite db file. Will be created if it does not exist.

Examples:

Load `file1.csv` and `file2.csv' files to sqlite database `test.db`

  $ qsv to sqlite test.db file1.csv file2.csv

Drop tables if they exist before loading.

  $ qsv to sqlite test.db --drop file1.csv file2.csv

Evolve tables if they exist. Read http://datapackage_convert.opendata.coop/evolve.html to explain how evolving is done.

  $ qsv to sqlite test.db --evolve file1.csv file2.csv


XLSX
Convert to new xlsx file.

Examples:

Load `file1.csv` and `file2.csv' into xlsx file

  $ qsv to xlsx output.xlsx file1.csv file2.csv

PARQUET
Convert to directory of parquet files.  Need to select a directory, it will be created if it does not exists.

Examples:

Convert `file1.csv` and `file2.csv' into `mydir/file1.parquet` and `mydir/file2.parquet` files.

  $ qsv to parquet mydir file1.csv file2.csv


DATAPACKAGE
Generate a datapackage, which contains stats and information about what is in the CSV files.

Examples:

Generate a `datapackage.json` file from `file1.csv` and `file2.csv' files.

  $ qsv to datapackage datapackage.json file1.csv file2.csv

Add more stats to datapackage.

  $ qsv to datapackage datapackage.json --stats file1.csv file2.csv

For all other conversions you can output the datapackage created by specifying `--print-package`.

  $ qsv to xlsx datapackage.xlsx --stats --print-package file1.csv file2.csv


Usage:
    qsv to postgres [options] <connection> [<input>...]
    qsv to sqlite [options] <sqlite> [<input>...]
    qsv to xlsx [options] <xlsx> [<input>...]
    qsv to parquet [options] <parquet> [<input>...]
    qsv to datapackage [options] <datapackage> [<input>...]
    qsv to --help

options:
    -k --print-package     Print statistics as datapackage, by default will print field summary.
    -a --stats             Produce extra statistics about the data beyond just type guessing.
    -c --stats-csv <path>  Output stats as CSV to specified file.
    -q --quiet             Do not print out field summary.
    -t --threads <num>     Use this amount of threads when calucating stats/type guessing.  
    -s --schema <arg>      The schema to load the data into. (postgres only)
    -d --drop              Drop tables before loading new data into them (postgres/sqlite only)
    -e --evolve            If loading into existing db, alter existing tables so that new data will load. (postgres/sqlite only)
    -p --seperator         For xlsx, use this character to help truncate xlsx sheet names, defaults to space.  
                           
Common options:
    -h, --help             Display this message
    -d, --delimiter <arg>  The field delimiter for reading CSV data.
                           Must be a single character. (default: ,)
"#;

use std::{io::Write, path::PathBuf};

use csvs_convert::{
    csvs_to_parquet_with_options, csvs_to_postgres_with_options, csvs_to_sqlite_with_options,
    csvs_to_xlsx_with_options, make_datapackage, DescribeOptions, Options,
};
use log::debug;
use serde::Deserialize;

use crate::{config::{Delimiter, self}, util, CliResult};

#[allow(dead_code)]
#[derive(Deserialize)]
struct Args {
    cmd_postgres:       bool,
    arg_connection:     Option<String>,
    cmd_sqlite:         bool,
    arg_sqlite:         Option<String>,
    cmd_parquet:        bool,
    arg_parquet:        Option<String>,
    cmd_xlsx:           bool,
    arg_xlsx:           Option<String>,
    cmd_datapackage:    bool,
    arg_datapackage:    Option<String>,
    arg_input:          Vec<PathBuf>,
    flag_delimiter:     Option<Delimiter>,
    flag_schema:        Option<String>,
    flag_seperator:     Option<String>,
    flag_drop:          bool,
    flag_evolve:        bool,
    flag_stats:         bool,
    flag_stats_csv:     Option<String>,
    flag_threads:       Option<usize>,
    flag_print_package: bool,
    flag_quiet:         bool,
}

pub fn run(argv: &[&str]) -> CliResult<()> {
    let args: Args = util::get_args(USAGE, argv)?;
    debug!("'to' ommand running");
    let options = Options::builder()
        .delimiter(args.flag_delimiter.map(config::Delimiter::as_byte))
        .schema(args.flag_schema.unwrap_or(String::new()))
        .seperator(args.flag_seperator.unwrap_or(" ".into()))
        .evolve(args.flag_evolve)
        .stats(args.flag_stats)
        .stats_csv(args.flag_stats_csv.unwrap_or(String::new()))
        .drop(args.flag_drop)
        .threads(args.flag_threads.unwrap_or(0))
        .build();

    let output;
    if args.cmd_postgres {
        debug!("converting to postgres");
        if args.arg_input.is_empty() {
            return fail_clierror!(
                "Need to add connection string as first argument then the input CSVs"
            );
        }
        output = csvs_to_postgres_with_options(
            args.arg_connection.expect("checked above"),
            args.arg_input,
            options,
        )?;
        debug!("convertion to postgres complete");
    } else if args.cmd_sqlite {
        debug!("converting to sqlite");
        if args.arg_input.is_empty() {
            return fail_clierror!(
                "Need to add the name of a sqlite db as first argument then the input CSVs"
            );
        }
        output = csvs_to_sqlite_with_options(
            args.arg_sqlite.expect("checked above"),
            args.arg_input,
            options,
        )?;
        debug!("convertion to xlsx complete");
    } else if args.cmd_parquet {
        debug!("converting to parquet");
        if args.arg_input.is_empty() {
            return fail_clierror!(
                "Need to add the directory of the parquet files as first argument then the input \
                 CSVs"
            );
        }
        output = csvs_to_parquet_with_options(
            args.arg_parquet.expect("checked above"),
            args.arg_input,
            options,
        )?;
        debug!("convertion to parquet complete");
    } else if args.cmd_xlsx {
        debug!("converting to xlsx");
        if args.arg_input.is_empty() {
            return fail_clierror!(
                "Need to add the name of a xlsx file as first argument then the input CSVs"
            );
        }
        output = csvs_to_xlsx_with_options(
            args.arg_xlsx.expect("checked above"),
            args.arg_input,
            options,
        )?;
        debug!("convertion to xlsx complete");
    } else if args.cmd_datapackage {
        debug!("creating datapackage");
        if args.arg_input.is_empty() {
            return fail_clierror!(
                "Need to add the name of a datapackage file as first argument then the input CSVs"
            );
        }
        let describe_options = DescribeOptions::builder()
            .delimiter(options.delimiter)
            .stats(options.stats)
            .threads(options.threads)
            .stats_csv(options.stats_csv);
        output = make_datapackage(args.arg_input, PathBuf::new(), &describe_options.build())?;
        let file = std::fs::File::create(args.arg_datapackage.expect("checked above"))?;
        serde_json::to_writer_pretty(file, &output)?;
        debug!("datapackage complete");
    } else {
        return fail_clierror!("Need to supply either xlsx,parquet,postgres,sqlite as command");
    }

    if args.flag_print_package {
        println!(
            "{}",
            serde_json::to_string_pretty(&output).expect("values should be serializable")
        );
    } else if !args.flag_quiet {
        let empty_array = vec![];
        for resource in output["resources"].as_array().unwrap_or(&empty_array) {
            let mut stdout = std::io::stdout();
            writeln!(&mut stdout)?;
            writeln!(
                &mut stdout,
                "Table '{}' ({} rows)",
                resource["name"].as_str().unwrap_or(""),
                resource["row_count"].as_i64().unwrap_or(0)
            )?;
            writeln!(&mut stdout)?;

            let mut tabwriter = tabwriter::TabWriter::new(stdout);

            writeln!(
                &mut tabwriter,
                "{}",
                ["Field Name", "Field Type", "Field Format"].join("\t")
            )?;

            for field in resource["schema"]["fields"]
                .as_array()
                .unwrap_or(&empty_array)
            {
                writeln!(
                    &mut tabwriter,
                    "{}",
                    [
                        field["name"].as_str().unwrap_or(""),
                        field["type"].as_str().unwrap_or(""),
                        field["format"].as_str().unwrap_or("")
                    ]
                    .join("\t")
                )?;
            }
            tabwriter.flush()?;
        }
        let mut stdout = std::io::stdout();
        writeln!(&mut stdout)?;
    }

    Ok(())
}