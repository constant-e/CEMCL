#include "EditAccDialog.hpp"
#include "EditAccDialogUI.hpp"

EditAccDialog::EditAccDialog(QWidget *parent)
    : QDialog(parent)
    , UI(new Ui::EditAccDialog) {
    UI->setupUi(this);
}

EditAccDialog::~EditAccDialog() {
    delete UI;
}
