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

class CEMCLUI
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

    void setupUI(QMainWindow *CEMCL)
    {
        // CEMCL
        CEMCL->resize(900, 600);

        // CWidget
        CWidget = new QWidget(CEMCL);

        // GLayout
        GLayout = new QGridLayout(CWidget);

        // Title
        Title = new QLabel(CWidget);
        QSizePolicy sizePolicy(QSizePolicy::Policy::Preferred, QSizePolicy::Policy::Preferred);
        sizePolicy.setHorizontalStretch(0);
        sizePolicy.setVerticalStretch(0);
        sizePolicy.setHeightForWidth(Title->sizePolicy().hasHeightForWidth());
        Title->setSizePolicy(sizePolicy);
        Title->setTextFormat(Qt::MarkdownText);
        Title->setAlignment(Qt::AlignLeading|Qt::AlignLeft|Qt::AlignVCenter);
        GLayout->addWidget(Title, 0, 0, 1, 1);

        // MainHLayout
        MainHLayout = new QHBoxLayout();

        // SettingsBtn
        SettingsBtn = new QPushButton(CWidget);
        QSizePolicy sizePolicy1(QSizePolicy::Policy::Fixed, QSizePolicy::Policy::Fixed);
        sizePolicy1.setHorizontalStretch(0);
        sizePolicy1.setVerticalStretch(0);
        sizePolicy1.setHeightForWidth(SettingsBtn->sizePolicy().hasHeightForWidth());
        SettingsBtn->setSizePolicy(sizePolicy1);
        SettingsBtn->setMinimumSize(QSize(100, 50));
        MainHLayout->addWidget(SettingsBtn);

        // HSpacer
        HSpacer = new QSpacerItem(40, 20, QSizePolicy::Policy::Expanding, QSizePolicy::Policy::Minimum);
        MainHLayout->addItem(HSpacer);
        
        // NewBtn
        NewBtn = new QPushButton(CWidget);
        sizePolicy1.setHeightForWidth(NewBtn->sizePolicy().hasHeightForWidth());
        NewBtn->setSizePolicy(sizePolicy1);
        NewBtn->setMinimumSize(QSize(100, 50));
        MainHLayout->addWidget(NewBtn);

        // EditBtn
        EditBtn = new QPushButton(CWidget);
        sizePolicy1.setHeightForWidth(EditBtn->sizePolicy().hasHeightForWidth());
        EditBtn->setSizePolicy(sizePolicy1);
        EditBtn->setMinimumSize(QSize(100, 50));
        MainHLayout->addWidget(EditBtn);

        // StartBtn
        StartBtn = new QPushButton(CWidget);
        sizePolicy1.setHeightForWidth(StartBtn->sizePolicy().hasHeightForWidth());
        StartBtn->setSizePolicy(sizePolicy1);
        StartBtn->setMinimumSize(QSize(100, 50));
        MainHLayout->addWidget(StartBtn);

        // MainHLayout End
        GLayout->addLayout(MainHLayout, 3, 0, 1, 1);

        // MainGLayout
        MainGLayout = new QGridLayout();

        // AccTableWidget
        AccTableWidget = new QTableWidget(CWidget);
        AccTableWidget->setColumnCount(2);
        AccTableWidget->setRowCount(1);
        QSizePolicy sizePolicy2(QSizePolicy::Policy::Preferred, QSizePolicy::Policy::Expanding);
        sizePolicy2.setHorizontalStretch(0);
        sizePolicy2.setVerticalStretch(0);
        sizePolicy2.setHeightForWidth(AccTableWidget->sizePolicy().hasHeightForWidth());
        AccTableWidget->setSizePolicy(sizePolicy2);
        AccTableWidget->setEditTriggers(QAbstractItemView::NoEditTriggers);
        AccTableWidget->setSelectionBehavior(QAbstractItemView::SelectRows);
        AccTableWidget->setSelectionMode(QAbstractItemView::SingleSelection);
        AccTableWidget->horizontalHeader()->setStretchLastSection(true);
        AccTableWidget->verticalHeader()->setVisible(false);
        MainGLayout->addWidget(AccTableWidget, 1, 0, 1, 1);

        // VerTableLayout
        VerTableWidget = new QTableWidget(CWidget);
        VerTableWidget->setColumnCount(3);
        VerTableWidget->setRowCount(1);
        VerTableWidget->setEditTriggers(QAbstractItemView::NoEditTriggers);
        VerTableWidget->setSelectionBehavior(QAbstractItemView::SelectRows);
        VerTableWidget->setSelectionMode(QAbstractItemView::SingleSelection);
        VerTableWidget->horizontalHeader()->setStretchLastSection(true);
        VerTableWidget->verticalHeader()->setVisible(false);
        MainGLayout->addWidget(VerTableWidget, 1, 1, 1, 1);

        // AccTitle
        AccTitle = new QLabel(CWidget);
        AccTitle->setTextFormat(Qt::MarkdownText);
        MainGLayout->addWidget(AccTitle, 0, 0, 1, 1);

        // VerTitle
        VerTitle = new QLabel(CWidget);
        VerTitle->setTextFormat(Qt::MarkdownText);
        MainGLayout->addWidget(VerTitle, 0, 1, 1, 1);

        // MainGLayout End
        GLayout->addLayout(MainGLayout, 1, 0, 1, 1);

        // GLayout End
        // CWidget End
        CEMCL->setCentralWidget(CWidget);

        // Strings
        retranslateUI(CEMCL);
    }

    void retranslateUI(QMainWindow *CEMCL)
    {
        // Set locale
        Title->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));
        SettingsBtn->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));
        NewBtn->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));
        EditBtn->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));
        StartBtn->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));
        AccTableWidget->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));
        VerTableWidget->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));
        AccTitle->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));
        VerTitle->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        CEMCL->setWindowTitle(QCoreApplication::translate("CEMCL", "CE Minecraft Launcher", nullptr));
        Title->setText(QCoreApplication::translate("CEMCL", "# CE Minecraft Launcher", nullptr));
        SettingsBtn->setText(QCoreApplication::translate("CEMCL", "Settings", nullptr));
        NewBtn->setText(QCoreApplication::translate("CEMCL", "New ", nullptr));
        EditBtn->setText(QCoreApplication::translate("CEMCL", "Edit", nullptr));
        StartBtn->setText(QCoreApplication::translate("CEMCL", "Start", nullptr));
        AccTitle->setText(QCoreApplication::translate("CEMCL", "## Account", nullptr));
        VerTitle->setText(QCoreApplication::translate("CEMCL", "## Minecraft Version", nullptr));
    }
};

QT_END_NAMESPACE
