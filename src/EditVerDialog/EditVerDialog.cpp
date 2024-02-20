#include "EditVerDialog.hpp"
#include "EditVerDialogUI.hpp"

EditVerDialog::EditVerDialog(QWidget *parent)
    : QDialog(parent)
    , UI(new Ui::EditVerDialog) {
    UI->setupUi(this);
}

EditVerDialog::~EditVerDialog() {
    delete UI;
}
