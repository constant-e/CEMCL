#pragma once

#include <QtCore/QLocale>
#include <QtCore/QVariant>
#include <QtWidgets/QApplication>
#include <QtWidgets/QDialog>
#include <QtWidgets/QGridLayout>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QLineEdit>
#include <QtWidgets/QPushButton>
#include <QtWidgets/QSpacerItem>
#include <QtWidgets/QTabWidget>
#include <QtWidgets/QVBoxLayout>
#include <QtWidgets/QWidget>

QT_BEGIN_NAMESPACE

class Ui_AddAccDialog
{
public:
    QVBoxLayout *MainVLayout;
    QLabel *Title;
    QTabWidget *TabWidget;
    QWidget *LegacyTab;
    QGridLayout *LegacyGLayout;
    QLabel *UUIDLabel;
    QLabel *NameLabel;
    QLineEdit *NameEdit;
    QLineEdit *UUIDEdit;
    QSpacerItem *VSpacer1;
    QWidget *MSTab;
    QGridLayout *MSGLayout;
    QLabel *PwdLabel;
    QLabel *AccLabel;
    QLineEdit *PwdEdit;
    QLineEdit *AccEdit;
    QSpacerItem *VSpacer2;
    QHBoxLayout *MainHLayout;
    QPushButton *CancelBtn;
    QPushButton *DoneBtn;

    void setupUi(QDialog *AddAccDialog)
    {
        if (AddAccDialog->objectName().isEmpty())
            AddAccDialog->setObjectName("AddAccDialog");
        AddAccDialog->resize(600, 300);
        AddAccDialog->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));
        MainVLayout = new QVBoxLayout(AddAccDialog);
        MainVLayout->setObjectName("MainVLayout");
        Title = new QLabel(AddAccDialog);
        Title->setObjectName("Title");
        Title->setTextFormat(Qt::MarkdownText);

        MainVLayout->addWidget(Title);

        TabWidget = new QTabWidget(AddAccDialog);
        TabWidget->setObjectName("TabWidget");
        LegacyTab = new QWidget();
        LegacyTab->setObjectName("LegacyTab");
        LegacyGLayout = new QGridLayout(LegacyTab);
        LegacyGLayout->setObjectName("LegacyGLayout");
        UUIDLabel = new QLabel(LegacyTab);
        UUIDLabel->setObjectName("UUIDLabel");

        LegacyGLayout->addWidget(UUIDLabel, 1, 0, 1, 1);

        NameLabel = new QLabel(LegacyTab);
        NameLabel->setObjectName("NameLabel");

        LegacyGLayout->addWidget(NameLabel, 0, 0, 1, 1);

        NameEdit = new QLineEdit(LegacyTab);
        NameEdit->setObjectName("NameEdit");

        LegacyGLayout->addWidget(NameEdit, 1, 1, 1, 1);

        UUIDEdit = new QLineEdit(LegacyTab);
        UUIDEdit->setObjectName("UUIDEdit");

        LegacyGLayout->addWidget(UUIDEdit, 0, 1, 1, 1);

        VSpacer1 = new QSpacerItem(20, 40, QSizePolicy::Policy::Minimum, QSizePolicy::Policy::Expanding);

        LegacyGLayout->addItem(VSpacer1, 2, 1, 1, 1);

        TabWidget->addTab(LegacyTab, QString());
        MSTab = new QWidget();
        MSTab->setObjectName("MSTab");
        MSGLayout = new QGridLayout(MSTab);
        MSGLayout->setObjectName("MSGLayout");
        PwdLabel = new QLabel(MSTab);
        PwdLabel->setObjectName("PwdLabel");

        MSGLayout->addWidget(PwdLabel, 1, 0, 1, 1);

        AccLabel = new QLabel(MSTab);
        AccLabel->setObjectName("AccLabel");

        MSGLayout->addWidget(AccLabel, 0, 0, 1, 1);

        PwdEdit = new QLineEdit(MSTab);
        PwdEdit->setObjectName("PwdEdit");

        MSGLayout->addWidget(PwdEdit, 1, 1, 1, 1);

        AccEdit = new QLineEdit(MSTab);
        AccEdit->setObjectName("AccEdit");

        MSGLayout->addWidget(AccEdit, 0, 1, 1, 1);

        VSpacer2 = new QSpacerItem(20, 40, QSizePolicy::Policy::Minimum, QSizePolicy::Policy::Expanding);

        MSGLayout->addItem(VSpacer2, 2, 1, 1, 1);

        TabWidget->addTab(MSTab, QString());

        MainVLayout->addWidget(TabWidget);

        MainHLayout = new QHBoxLayout();
        MainHLayout->setObjectName("MainHLayout");
        CancelBtn = new QPushButton(AddAccDialog);
        CancelBtn->setObjectName("CancelBtn");
        CancelBtn->setMinimumSize(QSize(100, 50));

        MainHLayout->addWidget(CancelBtn);

        DoneBtn = new QPushButton(AddAccDialog);
        DoneBtn->setObjectName("DoneBtn");
        DoneBtn->setMinimumSize(QSize(100, 50));

        MainHLayout->addWidget(DoneBtn);


        MainVLayout->addLayout(MainHLayout);


        retranslateUi(AddAccDialog);

        TabWidget->setCurrentIndex(0);


        QMetaObject::connectSlotsByName(AddAccDialog);
    } // setupUi

    void retranslateUi(QDialog *AddAccDialog)
    {
        AddAccDialog->setWindowTitle(QCoreApplication::translate("AddAccDialog", "Add a Account", nullptr));
        Title->setText(QCoreApplication::translate("AddAccDialog", "# Add a Account", nullptr));
        UUIDLabel->setText(QCoreApplication::translate("AddAccDialog", "UUID", nullptr));
        NameLabel->setText(QCoreApplication::translate("AddAccDialog", "User Name", nullptr));
        NameEdit->setPlaceholderText(QCoreApplication::translate("AddAccDialog", "Automatically generate", nullptr));
        UUIDEdit->setPlaceholderText(QCoreApplication::translate("AddAccDialog", "Steve", nullptr));
        TabWidget->setTabText(TabWidget->indexOf(LegacyTab), QCoreApplication::translate("AddAccDialog", "Offline ( Legacy ) Account", nullptr));
        PwdLabel->setText(QCoreApplication::translate("AddAccDialog", "Password", nullptr));
        AccLabel->setText(QCoreApplication::translate("AddAccDialog", "Account", nullptr));
        PwdEdit->setText(QString());
        TabWidget->setTabText(TabWidget->indexOf(MSTab), QCoreApplication::translate("AddAccDialog", "Official ( msa / Mojang ) Account", nullptr));
        CancelBtn->setText(QCoreApplication::translate("AddAccDialog", "Cancel", nullptr));
        DoneBtn->setText(QCoreApplication::translate("AddAccDialog", "Done", nullptr));
    } // retranslateUi

};

namespace Ui {
    class AddAccDialog: public Ui_AddAccDialog {};
} // namespace Ui

QT_END_NAMESPACE
