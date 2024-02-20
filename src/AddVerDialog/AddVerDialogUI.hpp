#pragma once

#include <QtCore/QLocale>
#include <QtCore/QVariant>
#include <QtWidgets/QApplication>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QDialog>
#include <QtWidgets/QGridLayout>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QLineEdit>
#include <QtWidgets/QListWidget>
#include <QtWidgets/QPushButton>
#include <QtWidgets/QSpinBox>
#include <QtWidgets/QToolButton>
#include <QtWidgets/QVBoxLayout>

QT_BEGIN_NAMESPACE

class Ui_AddVerDialog
{
public:
    QVBoxLayout *VLayout;
    QLabel *Title;
    QGridLayout *GLayout;
    QListWidget *MCList;
    QListWidget *OptfineList;
    QComboBox *ComboBox;
    QListWidget *ModList;
    QLabel *OptfineLabel;
    QLabel *MCLabel;
    QLabel *CfgLabel;
    QHBoxLayout *JavaHLayout;
    QLabel *JavaLabel;
    QLineEdit *JavaEdit;
    QToolButton *JavaBtn;
    QHBoxLayout *HeightHLayout;
    QLabel *HeightLabel;
    QSpinBox *HeightEdit;
    QHBoxLayout *WidthHLayout;
    QLabel *WidthLabel;
    QSpinBox *WidthEdit;
    QHBoxLayout *XmsHLayout;
    QLabel *XmsLabel;
    QLineEdit *XmsEdit;
    QHBoxLayout *XmxHLayout;
    QLabel *XmxLabel;
    QLineEdit *XmxEdit;
    QHBoxLayout *ArgsHLayout;
    QLabel *ArgsLabel;
    QLineEdit *ArgsEdit;
    QToolButton *ArgsButton;
    QHBoxLayout *MainHLayout;
    QPushButton *CancelBtn;
    QPushButton *DoneBtn;

    void setupUi(QDialog *AddVerDialog)
    {
        if (AddVerDialog->objectName().isEmpty())
            AddVerDialog->setObjectName("AddVerDialog");
        AddVerDialog->resize(900, 600);
        AddVerDialog->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));
        VLayout = new QVBoxLayout(AddVerDialog);
        VLayout->setObjectName("VLayout");
        Title = new QLabel(AddVerDialog);
        Title->setObjectName("Title");
        Title->setTextFormat(Qt::MarkdownText);

        VLayout->addWidget(Title);

        GLayout = new QGridLayout();
        GLayout->setObjectName("GLayout");
        MCList = new QListWidget(AddVerDialog);
        MCList->setObjectName("MCList");

        GLayout->addWidget(MCList, 1, 2, 1, 1);

        OptfineList = new QListWidget(AddVerDialog);
        OptfineList->setObjectName("OptfineList");

        GLayout->addWidget(OptfineList, 1, 4, 1, 1);

        ComboBox = new QComboBox(AddVerDialog);
        ComboBox->setObjectName("ComboBox");

        GLayout->addWidget(ComboBox, 0, 3, 1, 1);

        ModList = new QListWidget(AddVerDialog);
        ModList->setObjectName("ModList");

        GLayout->addWidget(ModList, 1, 3, 1, 1);

        OptfineLabel = new QLabel(AddVerDialog);
        OptfineLabel->setObjectName("OptfineLabel");
        OptfineLabel->setTextFormat(Qt::MarkdownText);

        GLayout->addWidget(OptfineLabel, 0, 4, 1, 1);

        MCLabel = new QLabel(AddVerDialog);
        MCLabel->setObjectName("MCLabel");
        MCLabel->setTextFormat(Qt::MarkdownText);

        GLayout->addWidget(MCLabel, 0, 2, 1, 1);


        VLayout->addLayout(GLayout);

        CfgLabel = new QLabel(AddVerDialog);
        CfgLabel->setObjectName("CfgLabel");
        CfgLabel->setTextFormat(Qt::MarkdownText);

        VLayout->addWidget(CfgLabel);

        JavaHLayout = new QHBoxLayout();
        JavaHLayout->setObjectName("JavaHLayout");
        JavaLabel = new QLabel(AddVerDialog);
        JavaLabel->setObjectName("JavaLabel");
        QSizePolicy sizePolicy(QSizePolicy::Policy::Preferred, QSizePolicy::Policy::Preferred);
        sizePolicy.setHorizontalStretch(0);
        sizePolicy.setVerticalStretch(0);
        sizePolicy.setHeightForWidth(JavaLabel->sizePolicy().hasHeightForWidth());
        JavaLabel->setSizePolicy(sizePolicy);
        JavaLabel->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        JavaHLayout->addWidget(JavaLabel);

        JavaEdit = new QLineEdit(AddVerDialog);
        JavaEdit->setObjectName("JavaEdit");
        sizePolicy.setHeightForWidth(JavaEdit->sizePolicy().hasHeightForWidth());
        JavaEdit->setSizePolicy(sizePolicy);
        JavaEdit->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        JavaHLayout->addWidget(JavaEdit);

        JavaBtn = new QToolButton(AddVerDialog);
        JavaBtn->setObjectName("JavaBtn");
        JavaBtn->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        JavaHLayout->addWidget(JavaBtn);


        VLayout->addLayout(JavaHLayout);

        HeightHLayout = new QHBoxLayout();
        HeightHLayout->setObjectName("HeightHLayout");
        HeightLabel = new QLabel(AddVerDialog);
        HeightLabel->setObjectName("HeightLabel");
        HeightLabel->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        HeightHLayout->addWidget(HeightLabel);

        HeightEdit = new QSpinBox(AddVerDialog);
        HeightEdit->setObjectName("HeightEdit");
        sizePolicy.setHeightForWidth(HeightEdit->sizePolicy().hasHeightForWidth());
        HeightEdit->setSizePolicy(sizePolicy);
        HeightEdit->setMinimum(100);
        HeightEdit->setMaximum(2147483647);
        HeightEdit->setValue(600);

        HeightHLayout->addWidget(HeightEdit);


        VLayout->addLayout(HeightHLayout);

        WidthHLayout = new QHBoxLayout();
        WidthHLayout->setObjectName("WidthHLayout");
        WidthLabel = new QLabel(AddVerDialog);
        WidthLabel->setObjectName("WidthLabel");
        WidthLabel->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        WidthHLayout->addWidget(WidthLabel);

        WidthEdit = new QSpinBox(AddVerDialog);
        WidthEdit->setObjectName("WidthEdit");
        sizePolicy.setHeightForWidth(WidthEdit->sizePolicy().hasHeightForWidth());
        WidthEdit->setSizePolicy(sizePolicy);
        WidthEdit->setMinimum(100);
        WidthEdit->setMaximum(2147483647);
        WidthEdit->setValue(800);

        WidthHLayout->addWidget(WidthEdit);


        VLayout->addLayout(WidthHLayout);

        XmsHLayout = new QHBoxLayout();
        XmsHLayout->setObjectName("XmsHLayout");
        XmsLabel = new QLabel(AddVerDialog);
        XmsLabel->setObjectName("XmsLabel");
        XmsLabel->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        XmsHLayout->addWidget(XmsLabel);

        XmsEdit = new QLineEdit(AddVerDialog);
        XmsEdit->setObjectName("XmsEdit");
        sizePolicy.setHeightForWidth(XmsEdit->sizePolicy().hasHeightForWidth());
        XmsEdit->setSizePolicy(sizePolicy);
        XmsEdit->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        XmsHLayout->addWidget(XmsEdit);


        VLayout->addLayout(XmsHLayout);

        XmxHLayout = new QHBoxLayout();
        XmxHLayout->setObjectName("XmxHLayout");
        XmxLabel = new QLabel(AddVerDialog);
        XmxLabel->setObjectName("XmxLabel");
        XmxLabel->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        XmxHLayout->addWidget(XmxLabel);

        XmxEdit = new QLineEdit(AddVerDialog);
        XmxEdit->setObjectName("XmxEdit");
        sizePolicy.setHeightForWidth(XmxEdit->sizePolicy().hasHeightForWidth());
        XmxEdit->setSizePolicy(sizePolicy);
        XmxEdit->setLocale(QLocale(QLocale::English, QLocale::UnitedStates));

        XmxHLayout->addWidget(XmxEdit);


        VLayout->addLayout(XmxHLayout);

        ArgsHLayout = new QHBoxLayout();
        ArgsHLayout->setObjectName("ArgsHLayout");
        ArgsLabel = new QLabel(AddVerDialog);
        ArgsLabel->setObjectName("ArgsLabel");

        ArgsHLayout->addWidget(ArgsLabel);

        ArgsEdit = new QLineEdit(AddVerDialog);
        ArgsEdit->setObjectName("ArgsEdit");

        ArgsHLayout->addWidget(ArgsEdit);

        ArgsButton = new QToolButton(AddVerDialog);
        ArgsButton->setObjectName("ArgsButton");

        ArgsHLayout->addWidget(ArgsButton);


        VLayout->addLayout(ArgsHLayout);

        MainHLayout = new QHBoxLayout();
        MainHLayout->setObjectName("MainHLayout");
        CancelBtn = new QPushButton(AddVerDialog);
        CancelBtn->setObjectName("CancelBtn");
        CancelBtn->setMinimumSize(QSize(100, 50));

        MainHLayout->addWidget(CancelBtn);

        DoneBtn = new QPushButton(AddVerDialog);
        DoneBtn->setObjectName("DoneBtn");
        DoneBtn->setMinimumSize(QSize(100, 50));

        MainHLayout->addWidget(DoneBtn);


        VLayout->addLayout(MainHLayout);


        retranslateUi(AddVerDialog);

        QMetaObject::connectSlotsByName(AddVerDialog);
    } // setupUi

    void retranslateUi(QDialog *AddVerDialog)
    {
        AddVerDialog->setWindowTitle(QCoreApplication::translate("AddVerDialog", "Add a Minecraft Version", nullptr));
        Title->setText(QCoreApplication::translate("AddVerDialog", "# Add a Minecraft Version", nullptr));
        ComboBox->setPlaceholderText(QCoreApplication::translate("AddVerDialog", "Mod Loader", nullptr));
        OptfineLabel->setText(QCoreApplication::translate("AddVerDialog", "### Optfine", nullptr));
        MCLabel->setText(QCoreApplication::translate("AddVerDialog", "### Minecraft Version", nullptr));
        CfgLabel->setText(QCoreApplication::translate("AddVerDialog", "### Configs", nullptr));
        JavaLabel->setText(QCoreApplication::translate("AddVerDialog", "Java Path", nullptr));
        JavaEdit->setPlaceholderText(QCoreApplication::translate("AddVerDialog", "java", nullptr));
        JavaBtn->setText(QCoreApplication::translate("AddVerDialog", "...", nullptr));
        HeightLabel->setText(QCoreApplication::translate("AddVerDialog", "Height", nullptr));
        WidthLabel->setText(QCoreApplication::translate("AddVerDialog", "Width", nullptr));
        XmsLabel->setText(QCoreApplication::translate("AddVerDialog", "Minimum Memory", nullptr));
        XmsEdit->setPlaceholderText(QCoreApplication::translate("AddVerDialog", "1G", nullptr));
        XmxLabel->setText(QCoreApplication::translate("AddVerDialog", "Maximum Memory", nullptr));
        XmxEdit->setPlaceholderText(QCoreApplication::translate("AddVerDialog", "2G", nullptr));
        ArgsLabel->setText(QCoreApplication::translate("AddVerDialog", "Launch Arguments ( Dangerous )", nullptr));
        ArgsButton->setText(QCoreApplication::translate("AddVerDialog", "...", nullptr));
        CancelBtn->setText(QCoreApplication::translate("AddVerDialog", "Cancel", nullptr));
        DoneBtn->setText(QCoreApplication::translate("AddVerDialog", "Done", nullptr));
    } // retranslateUi

};

namespace Ui {
    class AddVerDialog: public Ui_AddVerDialog {};
} // namespace Ui

QT_END_NAMESPACE
