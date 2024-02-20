#pragma once

#include <QtCore/QLocale>
#include <QtCore/QVariant>
#include <QtWidgets/QApplication>
#include <QtWidgets/QCheckBox>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QDialog>
#include <QtWidgets/QGridLayout>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QLineEdit>
#include <QtWidgets/QPushButton>
#include <QtWidgets/QScrollArea>
#include <QtWidgets/QSpacerItem>
#include <QtWidgets/QSpinBox>
#include <QtWidgets/QTabWidget>
#include <QtWidgets/QTextBrowser>
#include <QtWidgets/QToolButton>
#include <QtWidgets/QVBoxLayout>
#include <QtWidgets/QWidget>

QT_BEGIN_NAMESPACE

class Ui_Settings
{
public:
    QVBoxLayout *VLayout;
    QTabWidget *TabWidget;
    QWidget *SettingsTab;
    QVBoxLayout *verticalLayout;
    QLabel *SettingsTitle;
    QScrollArea *ScrollArea;
    QWidget *ScrollAreaWidgetContents;
    QVBoxLayout *verticalLayout_3;
    QLabel *GeneralTitle;
    QCheckBox *CloseCheckBox;
    QHBoxLayout *LangHLayout;
    QLabel *LangLabel;
    QComboBox *LangComboBox;
    QHBoxLayout *MCHLayout;
    QLabel *MCLabel;
    QLineEdit *MCEdit;
    QToolButton *MCBtn;
    QHBoxLayout *JavaHLayout;
    QLabel *JavaLabel;
    QLineEdit *JavaEdit;
    QToolButton *JavaBtn;
    QLabel *GameTitle;
    QHBoxLayout *XmsHLayout;
    QLabel *XmsLabel;
    QLineEdit *XmsEdit;
    QHBoxLayout *XmxHLayout;
    QLabel *XmxLabel;
    QLineEdit *XmxEdit;
    QHBoxLayout *HeightHLayout;
    QLabel *HeightLabel;
    QSpinBox *HeightEdit;
    QHBoxLayout *WidthHLayout;
    QLabel *WidthLabel;
    QSpinBox *WidthEdit;
    QLabel *DownloadTitle;
    QHBoxLayout *MCSourceHLayout;
    QLabel *MCSourceLabel;
    QLineEdit *MCSourceEdit;
    QHBoxLayout *ForgeSourceHLayout;
    QLabel *ForgeSourceLabel;
    QLineEdit *ForgeSourceEdit;
    QSpacerItem *VSpacer;
    QHBoxLayout *MainHLayout;
    QSpacerItem *HSpacer;
    QPushButton *CancelBtn;
    QPushButton *DoneBtn;
    QWidget *AboutTab;
    QGridLayout *AboutGLayout;
    QLabel *AboutTitle;
    QTextBrowser *AboutText;

    void setupUi(QDialog *Settings)
    {
        if (Settings->objectName().isEmpty())
            Settings->setObjectName("Settings");
        Settings->resize(900, 600);
        VLayout = new QVBoxLayout(Settings);
        VLayout->setObjectName("VLayout");
        TabWidget = new QTabWidget(Settings);
        TabWidget->setObjectName("TabWidget");
        TabWidget->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));
        SettingsTab = new QWidget();
        SettingsTab->setObjectName("SettingsTab");
        verticalLayout = new QVBoxLayout(SettingsTab);
        verticalLayout->setObjectName("verticalLayout");
        SettingsTitle = new QLabel(SettingsTab);
        SettingsTitle->setObjectName("SettingsTitle");
        QSizePolicy sizePolicy(QSizePolicy::Policy::Preferred, QSizePolicy::Policy::Preferred);
        sizePolicy.setHorizontalStretch(0);
        sizePolicy.setVerticalStretch(0);
        sizePolicy.setHeightForWidth(SettingsTitle->sizePolicy().hasHeightForWidth());
        SettingsTitle->setSizePolicy(sizePolicy);
        SettingsTitle->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));
        SettingsTitle->setTextFormat(Qt::MarkdownText);

        verticalLayout->addWidget(SettingsTitle);

        ScrollArea = new QScrollArea(SettingsTab);
        ScrollArea->setObjectName("ScrollArea");
        ScrollArea->setWidgetResizable(true);
        ScrollAreaWidgetContents = new QWidget();
        ScrollAreaWidgetContents->setObjectName("ScrollAreaWidgetContents");
        ScrollAreaWidgetContents->setGeometry(QRect(0, 0, 844, 537));
        verticalLayout_3 = new QVBoxLayout(ScrollAreaWidgetContents);
        verticalLayout_3->setObjectName("verticalLayout_3");
        GeneralTitle = new QLabel(ScrollAreaWidgetContents);
        GeneralTitle->setObjectName("GeneralTitle");
        sizePolicy.setHeightForWidth(GeneralTitle->sizePolicy().hasHeightForWidth());
        GeneralTitle->setSizePolicy(sizePolicy);
        GeneralTitle->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));
        GeneralTitle->setTextFormat(Qt::MarkdownText);

        verticalLayout_3->addWidget(GeneralTitle);

        CloseCheckBox = new QCheckBox(ScrollAreaWidgetContents);
        CloseCheckBox->setObjectName("CloseCheckBox");
        sizePolicy.setHeightForWidth(CloseCheckBox->sizePolicy().hasHeightForWidth());
        CloseCheckBox->setSizePolicy(sizePolicy);
        CloseCheckBox->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        verticalLayout_3->addWidget(CloseCheckBox);

        LangHLayout = new QHBoxLayout();
        LangHLayout->setObjectName("LangHLayout");
        LangLabel = new QLabel(ScrollAreaWidgetContents);
        LangLabel->setObjectName("LangLabel");
        sizePolicy.setHeightForWidth(LangLabel->sizePolicy().hasHeightForWidth());
        LangLabel->setSizePolicy(sizePolicy);
        LangLabel->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        LangHLayout->addWidget(LangLabel);

        LangComboBox = new QComboBox(ScrollAreaWidgetContents);
        LangComboBox->setObjectName("LangComboBox");

        LangHLayout->addWidget(LangComboBox);


        verticalLayout_3->addLayout(LangHLayout);

        MCHLayout = new QHBoxLayout();
        MCHLayout->setObjectName("MCHLayout");
        MCLabel = new QLabel(ScrollAreaWidgetContents);
        MCLabel->setObjectName("MCLabel");
        sizePolicy.setHeightForWidth(MCLabel->sizePolicy().hasHeightForWidth());
        MCLabel->setSizePolicy(sizePolicy);
        MCLabel->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        MCHLayout->addWidget(MCLabel);

        MCEdit = new QLineEdit(ScrollAreaWidgetContents);
        MCEdit->setObjectName("MCEdit");
        sizePolicy.setHeightForWidth(MCEdit->sizePolicy().hasHeightForWidth());
        MCEdit->setSizePolicy(sizePolicy);
        MCEdit->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        MCHLayout->addWidget(MCEdit);

        MCBtn = new QToolButton(ScrollAreaWidgetContents);
        MCBtn->setObjectName("MCBtn");
        MCBtn->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        MCHLayout->addWidget(MCBtn);


        verticalLayout_3->addLayout(MCHLayout);

        JavaHLayout = new QHBoxLayout();
        JavaHLayout->setObjectName("JavaHLayout");
        JavaLabel = new QLabel(ScrollAreaWidgetContents);
        JavaLabel->setObjectName("JavaLabel");
        sizePolicy.setHeightForWidth(JavaLabel->sizePolicy().hasHeightForWidth());
        JavaLabel->setSizePolicy(sizePolicy);
        JavaLabel->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        JavaHLayout->addWidget(JavaLabel);

        JavaEdit = new QLineEdit(ScrollAreaWidgetContents);
        JavaEdit->setObjectName("JavaEdit");
        sizePolicy.setHeightForWidth(JavaEdit->sizePolicy().hasHeightForWidth());
        JavaEdit->setSizePolicy(sizePolicy);
        JavaEdit->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        JavaHLayout->addWidget(JavaEdit);

        JavaBtn = new QToolButton(ScrollAreaWidgetContents);
        JavaBtn->setObjectName("JavaBtn");
        JavaBtn->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        JavaHLayout->addWidget(JavaBtn);


        verticalLayout_3->addLayout(JavaHLayout);

        GameTitle = new QLabel(ScrollAreaWidgetContents);
        GameTitle->setObjectName("GameTitle");
        sizePolicy.setHeightForWidth(GameTitle->sizePolicy().hasHeightForWidth());
        GameTitle->setSizePolicy(sizePolicy);
        GameTitle->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));
        GameTitle->setTextFormat(Qt::MarkdownText);

        verticalLayout_3->addWidget(GameTitle);

        XmsHLayout = new QHBoxLayout();
        XmsHLayout->setObjectName("XmsHLayout");
        XmsLabel = new QLabel(ScrollAreaWidgetContents);
        XmsLabel->setObjectName("XmsLabel");
        XmsLabel->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        XmsHLayout->addWidget(XmsLabel);

        XmsEdit = new QLineEdit(ScrollAreaWidgetContents);
        XmsEdit->setObjectName("XmsEdit");
        sizePolicy.setHeightForWidth(XmsEdit->sizePolicy().hasHeightForWidth());
        XmsEdit->setSizePolicy(sizePolicy);
        XmsEdit->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        XmsHLayout->addWidget(XmsEdit);


        verticalLayout_3->addLayout(XmsHLayout);

        XmxHLayout = new QHBoxLayout();
        XmxHLayout->setObjectName("XmxHLayout");
        XmxLabel = new QLabel(ScrollAreaWidgetContents);
        XmxLabel->setObjectName("XmxLabel");
        XmxLabel->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        XmxHLayout->addWidget(XmxLabel);

        XmxEdit = new QLineEdit(ScrollAreaWidgetContents);
        XmxEdit->setObjectName("XmxEdit");
        sizePolicy.setHeightForWidth(XmxEdit->sizePolicy().hasHeightForWidth());
        XmxEdit->setSizePolicy(sizePolicy);
        XmxEdit->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        XmxHLayout->addWidget(XmxEdit);


        verticalLayout_3->addLayout(XmxHLayout);

        HeightHLayout = new QHBoxLayout();
        HeightHLayout->setObjectName("HeightHLayout");
        HeightLabel = new QLabel(ScrollAreaWidgetContents);
        HeightLabel->setObjectName("HeightLabel");
        HeightLabel->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        HeightHLayout->addWidget(HeightLabel);

        HeightEdit = new QSpinBox(ScrollAreaWidgetContents);
        HeightEdit->setObjectName("HeightEdit");
        sizePolicy.setHeightForWidth(HeightEdit->sizePolicy().hasHeightForWidth());
        HeightEdit->setSizePolicy(sizePolicy);
        HeightEdit->setMinimum(100);
        HeightEdit->setMaximum(2147483647);
        HeightEdit->setValue(600);

        HeightHLayout->addWidget(HeightEdit);


        verticalLayout_3->addLayout(HeightHLayout);

        WidthHLayout = new QHBoxLayout();
        WidthHLayout->setObjectName("WidthHLayout");
        WidthLabel = new QLabel(ScrollAreaWidgetContents);
        WidthLabel->setObjectName("WidthLabel");
        WidthLabel->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        WidthHLayout->addWidget(WidthLabel);

        WidthEdit = new QSpinBox(ScrollAreaWidgetContents);
        WidthEdit->setObjectName("WidthEdit");
        sizePolicy.setHeightForWidth(WidthEdit->sizePolicy().hasHeightForWidth());
        WidthEdit->setSizePolicy(sizePolicy);
        WidthEdit->setMinimum(100);
        WidthEdit->setMaximum(2147483647);
        WidthEdit->setValue(800);

        WidthHLayout->addWidget(WidthEdit);


        verticalLayout_3->addLayout(WidthHLayout);

        DownloadTitle = new QLabel(ScrollAreaWidgetContents);
        DownloadTitle->setObjectName("DownloadTitle");
        sizePolicy.setHeightForWidth(DownloadTitle->sizePolicy().hasHeightForWidth());
        DownloadTitle->setSizePolicy(sizePolicy);
        DownloadTitle->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));
        DownloadTitle->setTextFormat(Qt::MarkdownText);

        verticalLayout_3->addWidget(DownloadTitle);

        MCSourceHLayout = new QHBoxLayout();
        MCSourceHLayout->setObjectName("MCSourceHLayout");
        MCSourceLabel = new QLabel(ScrollAreaWidgetContents);
        MCSourceLabel->setObjectName("MCSourceLabel");
        MCSourceLabel->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        MCSourceHLayout->addWidget(MCSourceLabel);

        MCSourceEdit = new QLineEdit(ScrollAreaWidgetContents);
        MCSourceEdit->setObjectName("MCSourceEdit");
        sizePolicy.setHeightForWidth(MCSourceEdit->sizePolicy().hasHeightForWidth());
        MCSourceEdit->setSizePolicy(sizePolicy);
        MCSourceEdit->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        MCSourceHLayout->addWidget(MCSourceEdit);


        verticalLayout_3->addLayout(MCSourceHLayout);

        ForgeSourceHLayout = new QHBoxLayout();
        ForgeSourceHLayout->setObjectName("ForgeSourceHLayout");
        ForgeSourceLabel = new QLabel(ScrollAreaWidgetContents);
        ForgeSourceLabel->setObjectName("ForgeSourceLabel");
        ForgeSourceLabel->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        ForgeSourceHLayout->addWidget(ForgeSourceLabel);

        ForgeSourceEdit = new QLineEdit(ScrollAreaWidgetContents);
        ForgeSourceEdit->setObjectName("ForgeSourceEdit");
        sizePolicy.setHeightForWidth(ForgeSourceEdit->sizePolicy().hasHeightForWidth());
        ForgeSourceEdit->setSizePolicy(sizePolicy);
        ForgeSourceEdit->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        ForgeSourceHLayout->addWidget(ForgeSourceEdit);


        verticalLayout_3->addLayout(ForgeSourceHLayout);

        VSpacer = new QSpacerItem(20, 40, QSizePolicy::Policy::Minimum, QSizePolicy::Policy::Expanding);

        verticalLayout_3->addItem(VSpacer);

        ScrollArea->setWidget(ScrollAreaWidgetContents);

        verticalLayout->addWidget(ScrollArea);

        MainHLayout = new QHBoxLayout();
        MainHLayout->setObjectName("MainHLayout");
        HSpacer = new QSpacerItem(40, 20, QSizePolicy::Policy::Expanding, QSizePolicy::Policy::Minimum);

        MainHLayout->addItem(HSpacer);

        CancelBtn = new QPushButton(SettingsTab);
        CancelBtn->setObjectName("CancelBtn");
        QSizePolicy sizePolicy1(QSizePolicy::Policy::Fixed, QSizePolicy::Policy::Fixed);
        sizePolicy1.setHorizontalStretch(0);
        sizePolicy1.setVerticalStretch(0);
        sizePolicy1.setHeightForWidth(CancelBtn->sizePolicy().hasHeightForWidth());
        CancelBtn->setSizePolicy(sizePolicy1);
        CancelBtn->setMinimumSize(QSize(100, 50));
        CancelBtn->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        MainHLayout->addWidget(CancelBtn);

        DoneBtn = new QPushButton(SettingsTab);
        DoneBtn->setObjectName("DoneBtn");
        sizePolicy1.setHeightForWidth(DoneBtn->sizePolicy().hasHeightForWidth());
        DoneBtn->setSizePolicy(sizePolicy1);
        DoneBtn->setMinimumSize(QSize(100, 50));
        DoneBtn->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        MainHLayout->addWidget(DoneBtn);


        verticalLayout->addLayout(MainHLayout);

        TabWidget->addTab(SettingsTab, QString());
        AboutTab = new QWidget();
        AboutTab->setObjectName("AboutTab");
        AboutGLayout = new QGridLayout(AboutTab);
        AboutGLayout->setObjectName("AboutGLayout");
        AboutTitle = new QLabel(AboutTab);
        AboutTitle->setObjectName("AboutTitle");
        AboutTitle->setTextFormat(Qt::MarkdownText);

        AboutGLayout->addWidget(AboutTitle, 0, 0, 1, 1);

        AboutText = new QTextBrowser(AboutTab);
        AboutText->setObjectName("AboutText");
        AboutText->setSource(QUrl(QString::fromUtf8("qrc:/text/about.md")));

        AboutGLayout->addWidget(AboutText, 1, 0, 1, 1);

        TabWidget->addTab(AboutTab, QString());

        VLayout->addWidget(TabWidget);


        retranslateUi(Settings);

        TabWidget->setCurrentIndex(0);


        QMetaObject::connectSlotsByName(Settings);
    } // setupUi

    void retranslateUi(QDialog *Settings)
    {
        Settings->setWindowTitle(QCoreApplication::translate("Settings", "Settings", nullptr));
        SettingsTitle->setText(QCoreApplication::translate("Settings", "# Settings", nullptr));
        GeneralTitle->setText(QCoreApplication::translate("Settings", "### General Settings", nullptr));
        CloseCheckBox->setText(QCoreApplication::translate("Settings", "Close Launcher After Starting Game", nullptr));
        LangLabel->setText(QCoreApplication::translate("Settings", "Language", nullptr));
        MCLabel->setText(QCoreApplication::translate("Settings", "Game Path", nullptr));
        MCEdit->setPlaceholderText(QCoreApplication::translate("Settings", ".minecraft", nullptr));
        MCBtn->setText(QCoreApplication::translate("Settings", "...", nullptr));
        JavaLabel->setText(QCoreApplication::translate("Settings", "Java Path", nullptr));
        JavaEdit->setPlaceholderText(QCoreApplication::translate("Settings", "java", nullptr));
        JavaBtn->setText(QCoreApplication::translate("Settings", "...", nullptr));
        GameTitle->setText(QCoreApplication::translate("Settings", "### Global Minecraft Settings", nullptr));
        XmsLabel->setText(QCoreApplication::translate("Settings", "Minimum Memory", nullptr));
        XmsEdit->setPlaceholderText(QCoreApplication::translate("Settings", "1G", nullptr));
        XmxLabel->setText(QCoreApplication::translate("Settings", "Maximum Memory", nullptr));
        XmxEdit->setPlaceholderText(QCoreApplication::translate("Settings", "2G", nullptr));
        HeightLabel->setText(QCoreApplication::translate("Settings", "Height", nullptr));
        WidthLabel->setText(QCoreApplication::translate("Settings", "Width", nullptr));
        DownloadTitle->setText(QCoreApplication::translate("Settings", "### Download Source Settings", nullptr));
        MCSourceLabel->setText(QCoreApplication::translate("Settings", "Minecraft Source", nullptr));
        MCSourceEdit->setPlaceholderText(QCoreApplication::translate("Settings", "https://piston-meta.mojang.com", nullptr));
        ForgeSourceLabel->setText(QCoreApplication::translate("Settings", "Forge Source", nullptr));
        ForgeSourceEdit->setPlaceholderText(QCoreApplication::translate("Settings", "https://maven.minecraftforge.net", nullptr));
        CancelBtn->setText(QCoreApplication::translate("Settings", "Cancel", nullptr));
        DoneBtn->setText(QCoreApplication::translate("Settings", "Done", nullptr));
        TabWidget->setTabText(TabWidget->indexOf(SettingsTab), QCoreApplication::translate("Settings", "Settings", nullptr));
        AboutTitle->setText(QCoreApplication::translate("Settings", "# About", nullptr));
        TabWidget->setTabText(TabWidget->indexOf(AboutTab), QCoreApplication::translate("Settings", "About", nullptr));
    } // retranslateUi

};

namespace Ui {
    class Settings: public Ui_Settings {};
} // namespace Ui

QT_END_NAMESPACE
