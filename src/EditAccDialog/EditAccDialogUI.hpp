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

class Ui_EditAccDialog
{
public:
    QVBoxLayout *VLayout;
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
    QHBoxLayout *HLayout;
    QPushButton *DelBtn;
    QSpacerItem *HSpacer;
    QPushButton *CancelBtn;
    QPushButton *DoneBtn;

    void setupUi(QDialog *EditAccDialog)
    {
        if (EditAccDialog->objectName().isEmpty())
            EditAccDialog->setObjectName("EditAccDialog");
        EditAccDialog->resize(600, 300);
        EditAccDialog->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));
        VLayout = new QVBoxLayout(EditAccDialog);
        VLayout->setObjectName("VLayout");
        Title = new QLabel(EditAccDialog);
        Title->setObjectName("Title");
        Title->setTextFormat(Qt::MarkdownText);

        VLayout->addWidget(Title);

        TabWidget = new QTabWidget(EditAccDialog);
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

        VLayout->addWidget(TabWidget);

        HLayout = new QHBoxLayout();
        HLayout->setObjectName("HLayout");
        DelBtn = new QPushButton(EditAccDialog);
        DelBtn->setObjectName("DelBtn");
        DelBtn->setMinimumSize(QSize(100, 50));

        HLayout->addWidget(DelBtn);

        HSpacer = new QSpacerItem(40, 20, QSizePolicy::Policy::Expanding, QSizePolicy::Policy::Minimum);

        HLayout->addItem(HSpacer);

        CancelBtn = new QPushButton(EditAccDialog);
        CancelBtn->setObjectName("CancelBtn");
        CancelBtn->setMinimumSize(QSize(100, 50));

        HLayout->addWidget(CancelBtn);

        DoneBtn = new QPushButton(EditAccDialog);
        DoneBtn->setObjectName("DoneBtn");
        DoneBtn->setMinimumSize(QSize(100, 50));

        HLayout->addWidget(DoneBtn);


        VLayout->addLayout(HLayout);


        retranslateUi(EditAccDialog);

        TabWidget->setCurrentIndex(0);


        QMetaObject::connectSlotsByName(EditAccDialog);
    } // setupUi

    void retranslateUi(QDialog *EditAccDialog)
    {
        EditAccDialog->setWindowTitle(QCoreApplication::translate("EditAccDialog", "Edit a Account", nullptr));
        Title->setText(QCoreApplication::translate("EditAccDialog", "# Edit a Account", nullptr));
        UUIDLabel->setText(QCoreApplication::translate("EditAccDialog", "UUID", nullptr));
        NameLabel->setText(QCoreApplication::translate("EditAccDialog", "User Name", nullptr));
        TabWidget->setTabText(TabWidget->indexOf(LegacyTab), QCoreApplication::translate("EditAccDialog", "Offline ( Legacy ) Account", nullptr));
        PwdLabel->setText(QCoreApplication::translate("EditAccDialog", "Password", nullptr));
        AccLabel->setText(QCoreApplication::translate("EditAccDialog", "Account", nullptr));
        PwdEdit->setText(QString());
        TabWidget->setTabText(TabWidget->indexOf(MSTab), QCoreApplication::translate("EditAccDialog", "Official ( msa / Mojang ) Account", nullptr));
        DelBtn->setText(QCoreApplication::translate("EditAccDialog", "Delete", nullptr));
        CancelBtn->setText(QCoreApplication::translate("EditAccDialog", "Cancel", nullptr));
        DoneBtn->setText(QCoreApplication::translate("EditAccDialog", "Done", nullptr));
    } // retranslateUi

};

namespace Ui {
    class EditAccDialog: public Ui_EditAccDialog {};
} // namespace Ui

QT_END_NAMESPACE
