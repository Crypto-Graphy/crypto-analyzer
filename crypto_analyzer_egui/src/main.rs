use std::sync::{Arc, Mutex};

use coinbase_transactions;
use coinbase_transactions::transaction_parser::CoinbaseTransactionRecord;
use csv_parser::CsvParser;
use csv_parser::{self, Csv};
use eframe::egui;
use im;

fn main() -> Result<(), eframe::Error> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(550.0, 320.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Crypto Graphy",
        options,
        Box::new(|_cc| Box::<CryptoGraphy>::default()),
    )
}

struct CryptoGraphy {
    file_path: String,
    coinbase_data: Arc<Mutex<im::Vector<CoinbaseTransactionRecord>>>,
    page: usize,
    rows: usize,
}

impl Default for CryptoGraphy {
    fn default() -> Self {
        Self {
            file_path: String::new(),
            coinbase_data: Arc::new(Mutex::new(im::Vector::default())),
            page: usize::default(),
            rows: 5usize,
        }
    }
}

impl eframe::App for CryptoGraphy {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Crypto Analyzer");
            ui.horizontal(|ui| {
                let label = ui.label("Select a csv file: ");
                ui.text_edit_singleline(&mut self.file_path)
                    .labelled_by(label.id);
            });

            if ui.button("Read File").clicked() {
                let file_path = &self.file_path;
                let data = std::fs::read_to_string(file_path).expect("Couldn't read file");
                let data: Vec<CoinbaseTransactionRecord> = Csv::parse_csv(&data);
                let data = im::Vector::from(data);
                match self.coinbase_data.lock() {
                    Ok(mut held_data) => *held_data = data,
                    Err(err) => panic!("{err}"),
                };
            }

            if self
                .coinbase_data
                .lock()
                .expect("Failed to get a lock on coinbase_data")
                .len()
                > 0usize
            {
                let _grid = egui::Grid::new("coinbase_data_table").show(ui, |ui| {
                    let transaction_type_label = ui.label("Transaction Type");
                    let currency_label = ui.label("Currency");
                    let amount_label = ui.label("Amount");
                    ui.end_row();

                    let data_clone = Arc::clone(&self.coinbase_data);
                    let data = data_clone
                        .lock()
                        .expect("Failed to get lock on coinbase_data");

                    let current_display = self.rows * self.page;
                    let mut display_data = Vec::new();

                    // let data_iter = data.iter().skip(current_display);
                    for i in current_display..(current_display + self.rows) {
                        display_data.push(data.get(i));
                    }

                    let display_data: im::Vector<&CoinbaseTransactionRecord> =
                        display_data.into_iter().flatten().collect();

                    // let data = im::Vector::from();
                    display_data.iter().for_each(|coinbase_transaction| {
                        ui.label(&coinbase_transaction.transaction_type)
                            .labelled_by(transaction_type_label.id);
                        ui.label(&coinbase_transaction.asset)
                            .labelled_by(currency_label.id);
                        ui.label(&coinbase_transaction.quantity_transacted.to_string())
                            .labelled_by(amount_label.id);
                        ui.end_row();
                    });

                    ui.horizontal(|ui| {
                        if ui.button("Previous").clicked() {
                            self.page = self.page - 1;
                        };
                        ui.label("Page");
                        ui.label((self.page + 1).to_string());
                        ui.label("/");
                        ui.label(calculate_total_pages(data.len(), self.rows).to_string());
                        if ui.button("Next").clicked() {
                            self.page = self.page + 1;
                        };
                    });
                });
            }
        });
    }
}

fn calculate_total_pages(data_length: usize, rows: usize) -> usize {
    if rows == 0 {
        return usize::MAX;
    };

    data_length / rows
}
