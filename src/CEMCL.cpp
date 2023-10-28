#include "CEMCL.hpp"
#include "ui_CEMCL.h"

#include "AddDialog/AddDialog.hpp"
#include "ConfigureDialog/ConfigureDialog.hpp"
#include "filestream/filestream.h"
#include "Settings/Settings.hpp"
#include "sonic/sonic.h"

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
        if (cfgText.empty()) {
            return loadVersionList(true);
        }
        Document doc;
        doc.Parse(cfgText);
        if (doc.HasParseError()) {
            return loadVersionList(true);
        }
        // TODO: if md5sum different {}
        if (!doc.HasMember("versionCount")) {
            return loadVersionList(true);
        }
        if (!doc.HasMember("versionList")) {
            return loadVersionList(true);
        }
        int count = doc.FindMember("versionCount")->value.GetInt64();
        Node * list = doc.AtPointer("versionList");
        versionList.resize(count);
        for (int i = 0; i < count; i++) {
            versionList[i].resize(4);
            versionList[i][0] = list->AtPointer(i)->FindMember("label")->value.GetString();
            versionList[i][1] = list->AtPointer(i)->FindMember("version")->value.GetString();
            versionList[i][2] = list->AtPointer(i)->FindMember("args")->value.GetString();
            versionList[i][3] = list->AtPointer(i)->FindMember("dir")->value.GetString();
        }
    } else {
        // TODO: create index.json from mc game path

        // for test only:
        versionList.resize(2);
        for (int i = 0; i < 2; i++) {
            versionList[i].resize(4);
            versionList[i][0] = "label";
            versionList[i][1] = "version";
            versionList[i][2] = "args";
            versionList[i][3] = "dir";
        }
    }
    int c = versionList.size();
    ui->tableWidget->setRowCount(c);
    ui->tableWidget->setHorizontalHeaderItem(0, new QTableWidgetItem("名称"));
    ui->tableWidget->setHorizontalHeaderItem(1, new QTableWidgetItem("版本"));
    ui->tableWidget->setHorizontalHeaderItem(2, new QTableWidgetItem("参数"));
    for (int i = 0; i < c; i++) {
        for (int j = 0; j < 3; j++) {
            QTableWidgetItem * item = new QTableWidgetItem();
            item->setText(QString(versionList[i][j].c_str()));
            ui->tableWidget->setItem(i, j, item);
        }
    }
    return true;
}

void CEMCL::onClickAddBtn()
{
    AddDialog * a = new AddDialog(this);
    a->show();
    a->exec();
}

void CEMCL::onClickConfigureBtn()
{
    ConfigureDialog * d = new ConfigureDialog(this);
    d->show();
    d->exec();
}

void CEMCL::onClickSettingsBtn()
{
    Settings * s = new Settings(this);
    s->show();
    s->exec();
}

void CEMCL::onClickStartBtn()
{
    /*
        minecraft launch args:
        -Xss1M
        -Djava.library.path=<.minecraft/bin/xxx>
        -Djna.tmpdir=<.minecraft/bin/xxx>
        -Dorg.lwjgl.system.SharedLibraryExtractPath=<.minecraft/bin/xxx>
        -Dio.netty.native.workdir=<.minecraft/bin/xxx>
        -Dminecraft.launcher.brand=CEMCL
        -Dminecraft.launcher.version=1.0.0b4
        -cp <文件>
        -Xmx2G
        -XX:+UnlockExperimentalVMOptions
        -XX:+UseG1GC
        -XX:G1NewSizePercent=20
        -XX:G1ReservePercent=20
        -XX:MaxGCPauseMillis=50
        -XX:G1HeapRegionSize=32M
        -Dlog4j.configurationFile=<.minecraft/assets/log_configs/xxx>
        net.minecraft.client.main.Main
        --username <username>
        --version <version>
        --gameDir <.minecraft>
        --assetsDir <.minecraft/assets>
        --assetIndex <index>
        --uuid <uuid>
        --accessToken <token>
        --clientId <client>
        --xuid <xuid>
        --userType msa
        --versionType release
        --quickPlayPath <path>
    */
    int c = ui->tableWidget->currentColumn();
    if (c == -1) return;
    string cmd = "java ";
    cmd.append(versionList[c][2]);
    system(cmd.c_str());
    cout << cmd;
}

CEMCL::CEMCL(QWidget *parent)
    : QMainWindow(parent)
    , ui(new Ui::CEMCL)
{
    if (!loadConfig()) return;
    ui->setupUi(this);
    if (!loadVersionList(false)) return;
    QObject::connect(ui->addButton, &QPushButton::clicked, this, &CEMCL::onClickAddBtn);
    QObject::connect(ui->configureBtn, &QPushButton::clicked, this, &CEMCL::onClickConfigureBtn);
    QObject::connect(ui->settingsBtn, &QPushButton::clicked, this, &CEMCL::onClickSettingsBtn);   
    QObject::connect(ui->startBtn, &QPushButton::clicked, this, &CEMCL::onClickStartBtn);
}

CEMCL::~CEMCL()
{
    delete ui;
}
