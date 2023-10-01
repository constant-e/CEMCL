#include "CEMCL.hpp"
#include "filestream/filestream.hpp"
#include "Settings/Settings.hpp"
#include "sonic/sonic.h"
#include "ui_CEMCL.h"

#include <QObject>
#include <QTableWidget>

#define DEFAULTCFG ""

bool CEMCL::loadConfig()
{
    string cfgText = openFile("config.json");
    if (cfgText.empty()) {
        saveFile("config.json", DEFAULTCFG);
    }

    return true;
}

void CEMCL::onClickAddBtn()
{

}

void CEMCL::onClickConfigureBtn()
{
    
}

void CEMCL::onClickSettingsBtn()
{
    Settings * s = new Settings(this);
    s->show();
    s->exec();
}

void CEMCL::onClickStartBtn()
{
    int c = ui->tableWidget->currentColumn();
    if (c == -1) return;
    string cmd = "javaw ";
    cmd.append(versionList[c][1]);
    system(cmd.c_str());
}

CEMCL::CEMCL(QWidget *parent)
    : QMainWindow(parent)
    , ui(new Ui::CEMCL)
{
    if (!loadConfig()) return;
    ui->setupUi(this);
    QObject::connect(ui->addButton, &QPushButton::clicked, this, &CEMCL::onClickAddBtn);
    QObject::connect(ui->configureBtn, &QPushButton::clicked, this, &CEMCL::onClickConfigureBtn);
    QObject::connect(ui->settingsBtn, &QPushButton::clicked, this, &CEMCL::onClickSettingsBtn);   
    QObject::connect(ui->startBtn, &QPushButton::clicked, this, &CEMCL::onClickStartBtn);
}

CEMCL::~CEMCL()
{
    delete ui;
}
