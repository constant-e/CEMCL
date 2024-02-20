#pragma once

#include <QtCore/QLocale>
#include <QtCore/QVariant>
#include <QtWidgets/QApplication>
#include <QtWidgets/QGridLayout>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QHeaderView>
#include <QtWidgets/QLabel>
#include <QtWidgets/QMainWindow>
#include <QtWidgets/QPushButton>
#include <QtWidgets/QSpacerItem>
#include <QtWidgets/QTableWidget>
#include <QtWidgets/QWidget>

QT_BEGIN_NAMESPACE

class Ui_CEMCL
{
public:
    QWidget *CWidget;
    QGridLayout *GLayout;
    QLabel *Title;
    QHBoxLayout *MainHLayout;
    QPushButton *SettingsBtn;
    QSpacerItem *HSpacer;
    QPushButton *NewBtn;
    QPushButton *EditBtn;
    QPushButton *StartBtn;
    QGridLayout *MainGLayout;
    QTableWidget *AccTableWidget;
    QTableWidget *VerTableWidget;
    QLabel *AccTitle;
    QLabel *VerTitle;

    void setupUi(QMainWindow *CEMCL)
    {
        if (CEMCL->objectName().isEmpty())
            CEMCL->setObjectName("CEMCL");
        CEMCL->resize(900, 600);
        CWidget = new QWidget(CEMCL);
        CWidget->setObjectName("CWidget");
        GLayout = new QGridLayout(CWidget);
        GLayout->setObjectName("GLayout");
        Title = new QLabel(CWidget);
        Title->setObjectName("Title");
        QSizePolicy sizePolicy(QSizePolicy::Policy::Preferred, QSizePolicy::Policy::Preferred);
        sizePolicy.setHorizontalStretch(0);
        sizePolicy.setVerticalStretch(0);
        sizePolicy.setHeightForWidth(Title->sizePolicy().hasHeightForWidth());
        Title->setSizePolicy(sizePolicy);
        Title->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));
        Title->setTextFormat(Qt::MarkdownText);
        Title->setAlignment(Qt::AlignLeading|Qt::AlignLeft|Qt::AlignVCenter);

        GLayout->addWidget(Title, 0, 0, 1, 1);

        MainHLayout = new QHBoxLayout();
        MainHLayout->setObjectName("MainHLayout");
        SettingsBtn = new QPushButton(CWidget);
        SettingsBtn->setObjectName("SettingsBtn");
        QSizePolicy sizePolicy1(QSizePolicy::Policy::Fixed, QSizePolicy::Policy::Fixed);
        sizePolicy1.setHorizontalStretch(0);
        sizePolicy1.setVerticalStretch(0);
        sizePolicy1.setHeightForWidth(SettingsBtn->sizePolicy().hasHeightForWidth());
        SettingsBtn->setSizePolicy(sizePolicy1);
        SettingsBtn->setMinimumSize(QSize(100, 50));
        SettingsBtn->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        MainHLayout->addWidget(SettingsBtn);

        HSpacer = new QSpacerItem(40, 20, QSizePolicy::Policy::Expanding, QSizePolicy::Policy::Minimum);

        MainHLayout->addItem(HSpacer);

        NewBtn = new QPushButton(CWidget);
        NewBtn->setObjectName("NewBtn");
        sizePolicy1.setHeightForWidth(NewBtn->sizePolicy().hasHeightForWidth());
        NewBtn->setSizePolicy(sizePolicy1);
        NewBtn->setMinimumSize(QSize(100, 50));
        NewBtn->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        MainHLayout->addWidget(NewBtn);

        EditBtn = new QPushButton(CWidget);
        EditBtn->setObjectName("EditBtn");
        sizePolicy1.setHeightForWidth(EditBtn->sizePolicy().hasHeightForWidth());
        EditBtn->setSizePolicy(sizePolicy1);
        EditBtn->setMinimumSize(QSize(100, 50));
        EditBtn->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        MainHLayout->addWidget(EditBtn);

        StartBtn = new QPushButton(CWidget);
        StartBtn->setObjectName("StartBtn");
        sizePolicy1.setHeightForWidth(StartBtn->sizePolicy().hasHeightForWidth());
        StartBtn->setSizePolicy(sizePolicy1);
        StartBtn->setMinimumSize(QSize(100, 50));
        StartBtn->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        MainHLayout->addWidget(StartBtn);


        GLayout->addLayout(MainHLayout, 3, 0, 1, 1);

        MainGLayout = new QGridLayout();
        MainGLayout->setObjectName("MainGLayout");
        AccTableWidget = new QTableWidget(CWidget);
        if (AccTableWidget->columnCount() < 2)
            AccTableWidget->setColumnCount(2);
        if (AccTableWidget->rowCount() < 1)
            AccTableWidget->setRowCount(1);
        AccTableWidget->setObjectName("AccTableWidget");
        QSizePolicy sizePolicy2(QSizePolicy::Policy::Preferred, QSizePolicy::Policy::Expanding);
        sizePolicy2.setHorizontalStretch(0);
        sizePolicy2.setVerticalStretch(0);
        sizePolicy2.setHeightForWidth(AccTableWidget->sizePolicy().hasHeightForWidth());
        AccTableWidget->setSizePolicy(sizePolicy2);
        AccTableWidget->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));
        AccTableWidget->setEditTriggers(QAbstractItemView::NoEditTriggers);
        AccTableWidget->setSelectionBehavior(QAbstractItemView::SelectRows);
        AccTableWidget->setSelectionMode(QAbstractItemView::SingleSelection);
        AccTableWidget->setRowCount(1);
        AccTableWidget->setColumnCount(2);
        AccTableWidget->horizontalHeader()->setStretchLastSection(true);
        AccTableWidget->verticalHeader()->setVisible(false);

        MainGLayout->addWidget(AccTableWidget, 1, 0, 1, 1);

        VerTableWidget = new QTableWidget(CWidget);
        if (VerTableWidget->columnCount() < 3)
            VerTableWidget->setColumnCount(3);
        if (VerTableWidget->rowCount() < 1)
            VerTableWidget->setRowCount(1);
        VerTableWidget->setObjectName("VerTableWidget");
        VerTableWidget->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));
        VerTableWidget->setEditTriggers(QAbstractItemView::NoEditTriggers);
        VerTableWidget->setSelectionBehavior(QAbstractItemView::SelectRows);
        VerTableWidget->setSelectionMode(QAbstractItemView::SingleSelection);
        VerTableWidget->setRowCount(1);
        VerTableWidget->setColumnCount(3);
        VerTableWidget->horizontalHeader()->setStretchLastSection(true);
        VerTableWidget->verticalHeader()->setVisible(false);

        MainGLayout->addWidget(VerTableWidget, 1, 1, 1, 1);

        AccTitle = new QLabel(CWidget);
        AccTitle->setObjectName("AccTitle");
        AccTitle->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));
        AccTitle->setTextFormat(Qt::MarkdownText);

        MainGLayout->addWidget(AccTitle, 0, 0, 1, 1);

        VerTitle = new QLabel(CWidget);
        VerTitle->setObjectName("VerTitle");
        VerTitle->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));
        VerTitle->setTextFormat(Qt::MarkdownText);

        MainGLayout->addWidget(VerTitle, 0, 1, 1, 1);


        GLayout->addLayout(MainGLayout, 1, 0, 1, 1);

        CEMCL->setCentralWidget(CWidget);

        retranslateUi(CEMCL);

        QMetaObject::connectSlotsByName(CEMCL);
    } // setupUi

    void retranslateUi(QMainWindow *CEMCL)
    {
        CEMCL->setWindowTitle(QCoreApplication::translate("CEMCL", "CE Minecraft Launcher", nullptr));
        Title->setText(QCoreApplication::translate("CEMCL", "# CE Minecraft Launcher", nullptr));
        SettingsBtn->setText(QCoreApplication::translate("CEMCL", "Settings", nullptr));
        NewBtn->setText(QCoreApplication::translate("CEMCL", "New ", nullptr));
        EditBtn->setText(QCoreApplication::translate("CEMCL", "Edit", nullptr));
        StartBtn->setText(QCoreApplication::translate("CEMCL", "Start", nullptr));
        AccTitle->setText(QCoreApplication::translate("CEMCL", "## Account", nullptr));
        VerTitle->setText(QCoreApplication::translate("CEMCL", "## Minecraft Version", nullptr));
    } // retranslateUi

};

namespace Ui {
    class CEMCL: public Ui_CEMCL {};
} // namespace Ui

QT_END_NAMESPACE
