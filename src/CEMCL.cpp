#include "CEMCL.hpp"
#include "filestream/filestream.hpp"
#include "Settings/Settings.hpp"
#include "sonic/sonic.h"
#include "ui_CEMCL.h"

#include <QObject>
#include <QTableWidget>

#define DEFAULTCFG "{\"account\":\"Steve\",\"gameDir\":\".minecraft\",\"javaDir\":\"\",\"token\": \"\"}"

using namespace sonic_json;

bool CEMCL::loadConfig()
{
    string cfgText = openFile("config.json");
    if (cfgText.empty()) {
        saveFile("config.json", DEFAULTCFG);
    } else {
        Document doc;
        doc.Parse(cfgText);
        if (doc.HasParseError()) {
            // dialog
            return false;
        }

        if (doc.HasMember("token")) {
            cfg.isOnline = !doc.FindMember("token")->value.Empty();
        } else {
            //dialog
        }

        if (cfg.isOnline) {
            // online
        } else {
            // offline
            if (doc.HasMember("account")) {
                cfg.account = doc.FindMember("account")->value.GetString();
            } else {
                // online
            }
        }
        if (doc.HasMember("gameDir")) {
            cfg.gameDir = doc.FindMember("gameDir")->value.GetString();
        } else {
            // online
        }
        
        if (doc.HasMember("javaDir")) {
            cfg.javaDir = doc.FindMember("javaDir")->value.GetString();
        } else {
            // online
        }
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
