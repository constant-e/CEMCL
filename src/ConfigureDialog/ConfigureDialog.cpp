#include "ConfigureDialog.hpp"
#include "ui_ConfigureDialog.h"

ConfigureDialog::ConfigureDialog(QWidget *parent)
    : QDialog(parent)
    , ui(new Ui::ConfigureDialog)
{
    ui->setupUi(this);
}

ConfigureDialog::~ConfigureDialog()
{
    delete ui;
}
