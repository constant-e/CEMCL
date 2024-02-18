#include "AddDialog.hpp"
#include "ui_AddDialog.h"

AddDialog::AddDialog(QWidget *parent)
    : QDialog(parent)
    , ui(new Ui::AddDialog) {
    ui->setupUi(this);
}

AddDialog::~AddDialog() {
    delete ui;
}
