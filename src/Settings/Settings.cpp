#include "Settings.hpp"
#include "SettingsUI.hpp"

Settings::Settings(QWidget *parent)
    : QDialog(parent)
    , UI(new Ui::Settings) {
    UI->setupUi(this);
}

Settings::~Settings() {
    delete UI;
}
