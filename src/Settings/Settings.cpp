#include "Settings.hpp"
#include "ui_Settings.h"

Settings::Settings(QWidget *parent)
    : QDialog(parent)
    , ui(new Ui::Settings) {
    ui->setupUi(this);
}

Settings::~Settings() {
    delete ui;
}
