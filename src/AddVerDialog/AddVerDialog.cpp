#include "AddVerDialog.hpp"
#include "AddVerDialogUI.hpp"

AddVerDialog::AddVerDialog(QWidget *parent)
    : QDialog(parent)
    , UI(new Ui::AddVerDialog) {
    UI->setupUi(this);
}

AddVerDialog::~AddVerDialog() {
    delete UI;
}
