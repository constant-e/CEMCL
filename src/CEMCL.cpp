#include "CEMCL.hpp"
#include "filestream/filestream.h"
#include "Settings/Settings.hpp"
#include "sonic/sonic.h"
#include "ui_CEMCL.h"

#include <QMessageBox>
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
            QMessageBox::warning(
                this, 
                "错误", 
                "加载配置文件错误：格式不正确。\n将使用默认配置文件。"); 
            return true;
        }

        if (doc.HasMember("token")) {
            cfg.isOnline = !doc.FindMember("token")->value.Empty();
        } else {
            QMessageBox::warning(
                this, 
                "异常", 
                "加载配置文件异常：不存在token。\n将使用离线模式。");
        }

        if (cfg.isOnline) {
            // online
        } else {
            // offline
            if (doc.HasMember("account")) {
                cfg.account = doc.FindMember("account")->value.GetString();
            } else {
                QMessageBox::warning(
                this, 
                "异常", 
                "加载配置文件异常：不存在account。\n将使用默认用户名。");
            }
        }

        if (doc.HasMember("gameDir")) {
            cfg.gameDir = doc.FindMember("gameDir")->value.GetString();
        } else {
            QMessageBox::warning(
                this, 
                "异常", 
                "加载配置文件异常：不存在gameDir。\n将使用默认值。");
        }
        
        if (doc.HasMember("javaDir")) {
            cfg.javaDir = doc.FindMember("javaDir")->value.GetString();
        } else {
            QMessageBox::warning(
                this, 
                "异常", 
                "加载配置文件异常：不存在javaDir。\n将使用默认值。");
        }
    }
    return true;
}

bool CEMCL::loadVersionList(bool ignoreIndexFile)
{
    if (!ignoreIndexFile) {
        string cfgText = openFile("index.json");
        if (!cfgText.empty()) {
            Document doc;
            doc.Parse(cfgText);
            if (doc.HasParseError()) {
                return loadVersionList(true);
            }
            // if md5sum different
            return true;
        }
        return loadVersionList(true);
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
    if (!loadVersionList(false)) return;
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
