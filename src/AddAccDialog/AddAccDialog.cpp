#include "AddAccDialog.hpp"
#include "AddAccDialogUI.hpp"

AddAccDialog::AddAccDialog(QWidget *parent)
    : QDialog(parent)
    , UI(new Ui::AddAccDialog) {
    UI->setupUi(this);
}

AddAccDialog::~AddAccDialog() {
    delete UI;
}
