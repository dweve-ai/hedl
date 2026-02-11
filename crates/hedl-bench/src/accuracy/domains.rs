// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! 8 Domain-Specific Dataset Generators
//!
//! Provides diverse, realistic datasets across multiple industries for comprehensive
//! LLM accuracy testing. Each domain has unique characteristics that test different
//! aspects of data comprehension.

use crate::accuracy::complexity::{ComplexityLevel, ComplexityProfile};

/// Domain categories for benchmark datasets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Domain {
    /// Financial data: transactions, portfolios, markets
    Finance,
    /// Medical records, patients, treatments, diagnoses
    Healthcare,
    /// Legal documents, cases, contracts, regulations
    Legal,
    /// Products, orders, customers, inventory
    Ecommerce,
    /// Sensor readings, devices, metrics, alerts
    IoT,
    /// Experiments, measurements, publications, citations
    Scientific,
    /// Organization hierarchy, employees, projects
    Enterprise,
    /// Users, posts, comments, social graphs
    SocialMedia,
}

impl Domain {
    /// All domains for iteration
    pub const ALL: [Domain; 8] = [
        Domain::Finance,
        Domain::Healthcare,
        Domain::Legal,
        Domain::Ecommerce,
        Domain::IoT,
        Domain::Scientific,
        Domain::Enterprise,
        Domain::SocialMedia,
    ];

    /// Human-readable name
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Domain::Finance => "Finance",
            Domain::Healthcare => "Healthcare",
            Domain::Legal => "Legal",
            Domain::Ecommerce => "E-commerce",
            Domain::IoT => "IoT",
            Domain::Scientific => "Scientific",
            Domain::Enterprise => "Enterprise",
            Domain::SocialMedia => "Social Media",
        }
    }

    /// Short code for IDs
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Domain::Finance => "fin",
            Domain::Healthcare => "med",
            Domain::Legal => "law",
            Domain::Ecommerce => "eco",
            Domain::IoT => "iot",
            Domain::Scientific => "sci",
            Domain::Enterprise => "ent",
            Domain::SocialMedia => "soc",
        }
    }

    /// Description of domain characteristics
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            Domain::Finance => {
                "Financial transactions, portfolios, market data. Features: precise decimals, \
                 currency codes, temporal sequences, regulatory compliance fields."
            }
            Domain::Healthcare => {
                "Medical records, patient data, treatments. Features: coded diagnoses (ICD-10), \
                 dosage calculations, temporal treatments, privacy-sensitive fields."
            }
            Domain::Legal => {
                "Legal documents, case files, contracts. Features: long text, references to \
                 statutes, nested clause structures, date-sensitive terms."
            }
            Domain::Ecommerce => {
                "Products, orders, inventory. Features: nested line items, price calculations, \
                 SKU codes, inventory counts, customer relationships."
            }
            Domain::IoT => {
                "Sensor data, device metrics, alerts. Features: high-frequency timestamps, \
                 tensor data (sensor arrays), threshold alerts, device hierarchies."
            }
            Domain::Scientific => {
                "Research data, experiments, publications. Features: measurement precision, \
                 citation networks, statistical values, multi-author attributions."
            }
            Domain::Enterprise => {
                "Organization structure, employees, projects. Features: deep hierarchies \
                 (company>division>dept>team>employee), cross-references, budget tracking."
            }
            Domain::SocialMedia => {
                "Users, posts, interactions. Features: follower graphs, engagement metrics, \
                 timestamp sequences, content moderation flags, viral propagation."
            }
        }
    }

    /// Primary data characteristics
    #[must_use]
    pub fn characteristics(&self) -> DomainCharacteristics {
        match self {
            Domain::Finance => DomainCharacteristics {
                primary_types: vec![
                    "Transaction",
                    "Account",
                    "Portfolio",
                    "Position",
                    "Currency",
                ],
                typical_nesting: 3,
                has_precise_numbers: true,
                has_temporal_sequences: true,
                has_cross_references: true,
                has_regulatory_fields: true,
                numeric_precision: 4,
                typical_entity_count: 500,
            },
            Domain::Healthcare => DomainCharacteristics {
                primary_types: vec![
                    "Patient",
                    "Diagnosis",
                    "Treatment",
                    "Medication",
                    "Provider",
                ],
                typical_nesting: 4,
                has_precise_numbers: true,
                has_temporal_sequences: true,
                has_cross_references: true,
                has_regulatory_fields: true,
                numeric_precision: 3,
                typical_entity_count: 200,
            },
            Domain::Legal => DomainCharacteristics {
                primary_types: vec!["Case", "Contract", "Clause", "Party", "Citation"],
                typical_nesting: 5,
                has_precise_numbers: false,
                has_temporal_sequences: true,
                has_cross_references: true,
                has_regulatory_fields: true,
                numeric_precision: 2,
                typical_entity_count: 100,
            },
            Domain::Ecommerce => DomainCharacteristics {
                primary_types: vec!["Product", "Order", "Customer", "LineItem", "Category"],
                typical_nesting: 3,
                has_precise_numbers: true,
                has_temporal_sequences: true,
                has_cross_references: true,
                has_regulatory_fields: false,
                numeric_precision: 2,
                typical_entity_count: 1000,
            },
            Domain::IoT => DomainCharacteristics {
                primary_types: vec!["Device", "Sensor", "Reading", "Alert", "Metric"],
                typical_nesting: 2,
                has_precise_numbers: true,
                has_temporal_sequences: true,
                has_cross_references: true,
                has_regulatory_fields: false,
                numeric_precision: 6,
                typical_entity_count: 5000,
            },
            Domain::Scientific => DomainCharacteristics {
                primary_types: vec![
                    "Experiment",
                    "Measurement",
                    "Publication",
                    "Author",
                    "Dataset",
                ],
                typical_nesting: 3,
                has_precise_numbers: true,
                has_temporal_sequences: true,
                has_cross_references: true,
                has_regulatory_fields: false,
                numeric_precision: 8,
                typical_entity_count: 300,
            },
            Domain::Enterprise => DomainCharacteristics {
                primary_types: vec![
                    "Company",
                    "Division",
                    "Department",
                    "Team",
                    "Employee",
                    "Project",
                ],
                typical_nesting: 5,
                has_precise_numbers: true,
                has_temporal_sequences: true,
                has_cross_references: true,
                has_regulatory_fields: false,
                numeric_precision: 2,
                typical_entity_count: 500,
            },
            Domain::SocialMedia => DomainCharacteristics {
                primary_types: vec!["User", "Post", "Comment", "Like", "Follow", "Share"],
                typical_nesting: 3,
                has_precise_numbers: false,
                has_temporal_sequences: true,
                has_cross_references: true,
                has_regulatory_fields: false,
                numeric_precision: 0,
                typical_entity_count: 2000,
            },
        }
    }

    /// Get recommended complexity profile for this domain
    #[must_use]
    pub fn default_profile(&self, level: ComplexityLevel) -> ComplexityProfile {
        let chars = self.characteristics();
        let (min_entities, max_entities) = level.entity_range();
        let scale_factor = chars.typical_entity_count as f64 / 500.0;

        let mut profile = ComplexityProfile::new(level)
            .with_entities(((min_entities + max_entities) as f64 / 2.0 * scale_factor) as usize)
            .with_nesting(chars.typical_nesting.min(level.nesting_depth().1));

        if chars.has_cross_references && level >= ComplexityLevel::L3Intermediate {
            profile = profile.with_references();
        }
        if chars.has_temporal_sequences {
            profile = profile.with_temporal();
        }
        if chars.has_precise_numbers {
            profile = profile.with_tensors();
        }

        profile
    }
}

impl std::fmt::Display for Domain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Characteristics of a domain's data
#[derive(Debug, Clone)]
pub struct DomainCharacteristics {
    /// Primary entity types in this domain
    pub primary_types: Vec<&'static str>,
    /// Typical nesting depth
    pub typical_nesting: usize,
    /// Whether domain requires precise decimal numbers
    pub has_precise_numbers: bool,
    /// Whether domain has temporal sequences
    pub has_temporal_sequences: bool,
    /// Whether domain has cross-references between entities
    pub has_cross_references: bool,
    /// Whether domain has regulatory/compliance fields
    pub has_regulatory_fields: bool,
    /// Typical numeric precision (decimal places)
    pub numeric_precision: u8,
    /// Typical entity count for realistic datasets
    pub typical_entity_count: usize,
}

/// A domain-specific dataset with HEDL representation
#[derive(Debug, Clone)]
pub struct DomainDataset {
    /// Unique dataset ID
    pub id: String,
    /// Domain this dataset belongs to
    pub domain: Domain,
    /// Complexity profile
    pub profile: ComplexityProfile,
    /// HEDL source data
    pub hedl: String,
    /// Dataset description
    pub description: String,
    /// Seed used for generation (for reproducibility)
    pub seed: u64,
}

impl DomainDataset {
    /// Create a new domain dataset
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        domain: Domain,
        level: ComplexityLevel,
        hedl: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            domain,
            profile: domain.default_profile(level),
            hedl: hedl.into(),
            description: String::new(),
            seed: 0,
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set seed for reproducibility
    #[must_use]
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Set custom profile
    #[must_use]
    pub fn with_profile(mut self, profile: ComplexityProfile) -> Self {
        self.profile = profile;
        self
    }

    /// Get complexity level
    #[must_use]
    pub fn complexity_level(&self) -> ComplexityLevel {
        self.profile.level
    }

    /// Validate dataset matches declared profile
    #[must_use]
    pub fn validate(&self) -> Vec<String> {
        let mut issues = self.profile.validate();

        if self.hedl.is_empty() {
            issues.push("HEDL content is empty".to_string());
        }

        if self.id.is_empty() {
            issues.push("Dataset ID is empty".to_string());
        }

        issues
    }
}

/// Generator for domain-specific datasets
pub struct DomainDatasetGenerator {
    /// Random seed for reproducibility
    seed: u64,
}

impl DomainDatasetGenerator {
    /// Create a new generator with a seed
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// Generate a dataset for a specific domain and complexity
    #[must_use]
    pub fn generate(&self, domain: Domain, level: ComplexityLevel) -> DomainDataset {
        let id = format!(
            "{}_{}_{}",
            domain.code(),
            level.code().to_lowercase(),
            self.seed
        );

        let hedl = self.generate_hedl(domain, level);

        DomainDataset::new(id, domain, level, hedl)
            .with_seed(self.seed)
            .with_description(format!(
                "{} dataset at {} complexity",
                domain.name(),
                level.name()
            ))
    }

    /// Generate HEDL content for a domain and level
    fn generate_hedl(&self, domain: Domain, level: ComplexityLevel) -> String {
        // Get appropriate entity count for level
        let (min, max) = level.entity_range();
        let count = (min + max) / 2;

        match domain {
            Domain::Finance => self.generate_finance(count, level),
            Domain::Healthcare => self.generate_healthcare(count, level),
            Domain::Legal => self.generate_legal(count, level),
            Domain::Ecommerce => self.generate_ecommerce(count, level),
            Domain::IoT => self.generate_iot(count, level),
            Domain::Scientific => self.generate_scientific(count, level),
            Domain::Enterprise => self.generate_enterprise(count, level),
            Domain::SocialMedia => self.generate_social_media(count, level),
        }
    }

    fn generate_finance(&self, count: usize, level: ComplexityLevel) -> String {
        let nesting = if level >= ComplexityLevel::L3Intermediate {
            "%N:Portfolio>Position"
        } else {
            ""
        };

        let refs = if level >= ComplexityLevel::L4Advanced {
            "@Currency:currency"
        } else {
            "USD"
        };

        format!(
            r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Currency:[code,name,symbol,exchange_rate]
%S:Account:[id,holder,type,balance,currency,opened_at]
%S:Transaction:[id,account,type,amount,currency,timestamp,status]
%S:Portfolio:[id,account,name,value,risk_level]
%S:Position:[id,symbol,shares,purchase_price,current_price]
{}
---
currencies:@Currency
 |USD,US Dollar,$,1.0000
 |EUR,Euro,€,1.0842
 |GBP,British Pound,£,1.2654
 |JPY,Japanese Yen,¥,0.0067
accounts:@Account
{}
transactions:@Transaction
{}
"#,
            nesting,
            self.generate_finance_accounts(count, refs),
            self.generate_finance_transactions(count.min(50))
        )
    }

    fn generate_finance_accounts(&self, count: usize, currency_ref: &str) -> String {
        let mut lines = String::new();
        let types = ["checking", "savings", "investment", "retirement"];
        let names = [
            "Alice Chen",
            "Bob Kumar",
            "Carol Smith",
            "David Lee",
            "Eve Wilson",
        ];

        for i in 1..=count.min(100) {
            let holder = names[i % names.len()];
            let acc_type = types[i % types.len()];
            let balance = 1000.0 + (i as f64 * 123.45) % 100000.0;
            let year = 2020 + (i % 5);
            let month = 1 + (i % 12);

            lines.push_str(&format!(
                " |acc{:03},{},{},{:.2},{},{}-{:02}-15\n",
                i, holder, acc_type, balance, currency_ref, year, month
            ));
        }
        lines
    }

    fn generate_finance_transactions(&self, count: usize) -> String {
        let mut lines = String::new();
        let types = ["deposit", "withdrawal", "transfer", "payment", "refund"];
        let statuses = ["completed", "pending", "failed"];

        for i in 1..=count {
            let tx_type = types[i % types.len()];
            let status = statuses[i % statuses.len()];
            let amount = 10.0 + (i as f64 * 17.89) % 5000.0;
            let acc = 1 + (i % 10);
            let day = 1 + (i % 28);
            let hour = i % 24;

            lines.push_str(&format!(
                " |tx{:04},@Account:acc{:03},{},{:.2},USD,2024-12-{:02}T{:02}:30:00Z,{}\n",
                i, acc, tx_type, amount, day, hour, status
            ));
        }
        lines
    }

    fn generate_healthcare(&self, count: usize, level: ComplexityLevel) -> String {
        let nesting = if level >= ComplexityLevel::L3Intermediate {
            "%N:Patient>Diagnosis\n%N:Patient>Treatment"
        } else {
            ""
        };

        format!(
            r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Provider:[id,name,specialty,license,hospital]
%S:Patient:[id,name,dob,gender,blood_type,allergies]
%S:Diagnosis:[id,code,description,severity,diagnosed_at]
%S:Treatment:[id,diagnosis,medication,dosage,frequency,start_date,end_date]
%S:Medication:[id,name,class,contraindications]
{}
---
providers:@Provider
 |dr001,Dr. Sarah Johnson,Cardiology,MD12345,Memorial General
 |dr002,Dr. Michael Chen,Oncology,MD12346,Memorial General
 |dr003,Dr. Emily Williams,Neurology,MD12347,City Medical Center
medications:@Medication
 |med001,Aspirin,NSAID,[bleeding disorders]
 |med002,Metformin,Antidiabetic,[kidney disease]
 |med003,Lisinopril,ACE Inhibitor,[pregnancy]
patients:@Patient
{}
"#,
            nesting,
            self.generate_healthcare_patients(count, level)
        )
    }

    fn generate_healthcare_patients(&self, count: usize, level: ComplexityLevel) -> String {
        let mut lines = String::new();
        let names = [
            "John Smith",
            "Maria Garcia",
            "James Wilson",
            "Sarah Brown",
            "Robert Taylor",
        ];
        let blood_types = ["A+", "A-", "B+", "B-", "O+", "O-", "AB+", "AB-"];
        let genders = ["M", "F"];
        let allergies = ["none", "penicillin", "sulfa", "latex", "iodine"];

        for i in 1..=count.min(50) {
            let name = names[i % names.len()];
            let blood = blood_types[i % blood_types.len()];
            let gender = genders[i % genders.len()];
            let allergy = allergies[i % allergies.len()];
            let year = 1950 + (i % 60);
            let month = 1 + (i % 12);
            let day = 1 + (i % 28);

            lines.push_str(&format!(
                " |pat{:03},{},{}-{:02}-{:02},{},{},{}\n",
                i, name, year, month, day, gender, blood, allergy
            ));

            // Add nested diagnoses and treatments for higher complexity
            if level >= ComplexityLevel::L3Intermediate && i <= 10 {
                let codes = ["I25.10", "E11.9", "J45.909", "G43.909"];
                let code = codes[i % codes.len()];
                lines.push_str(&format!(
                    " |diag{:03},{},Chronic condition,moderate,2024-06-15\n",
                    i, code
                ));
                lines.push_str(&format!(
                    " |treat{:03},@Diagnosis:diag{:03},@Medication:med{:03},10mg,daily,2024-06-15,~\n",
                    i,
                    i,
                    1 + (i % 3)
                ));
            }
        }
        lines
    }

    fn generate_legal(&self, count: usize, _level: ComplexityLevel) -> String {
        format!(
            r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Party:[id,name,type,jurisdiction,registered_at]
%S:Case:[id,title,type,status,filed_at,court]
%S:Contract:[id,title,parties,effective_date,expiration_date,value]
%S:Clause:[id,contract,section,title,text]
%N:Contract>Clause
---
parties:@Party
 |party001,Acme Corporation,corporation,Delaware,2010-03-15
 |party002,John Doe,individual,California,~
 |party003,Smith & Associates,partnership,New York,2015-07-22
cases:@Case
{}
contracts:@Contract
{}
"#,
            self.generate_legal_cases(count.min(20)),
            self.generate_legal_contracts(count.min(10))
        )
    }

    fn generate_legal_cases(&self, count: usize) -> String {
        let mut lines = String::new();
        let types = ["civil", "criminal", "contract", "tort", "administrative"];
        let statuses = ["open", "closed", "pending", "appealed"];
        let courts = [
            "District Court",
            "Superior Court",
            "Appeals Court",
            "Supreme Court",
        ];

        for i in 1..=count {
            let case_type = types[i % types.len()];
            let status = statuses[i % statuses.len()];
            let court = courts[i % courts.len()];
            let month = 1 + (i % 12);
            let day = 1 + (i % 28);

            lines.push_str(&format!(
                " |case{:03},Case #{:03} v. Defendant,{},{},2024-{:02}-{:02},{}\n",
                i, i, case_type, status, month, day, court
            ));
        }
        lines
    }

    fn generate_legal_contracts(&self, count: usize) -> String {
        let mut lines = String::new();
        let titles = [
            "Service Agreement",
            "License Agreement",
            "NDA",
            "Employment Contract",
            "Lease Agreement",
        ];

        for i in 1..=count {
            let title = titles[i % titles.len()];
            let value = 10000 * (i % 100 + 1);
            let month = 1 + (i % 12);

            lines.push_str(&format!(
                " |cont{:03},{},[@Party:party001, @Party:party002],2024-{:02}-01,2025-{:02}-01,{}\n",
                i, title, month, month, value
            ));

            // Add clauses
            for j in 1..=3 {
                lines.push_str(&format!(
                    " |clause{:03}{},@Contract:cont{:03},Section {}.{},Clause Title,The parties agree to the terms herein.\n",
                    i, j, i, i, j
                ));
            }
        }
        lines
    }

    fn generate_ecommerce(&self, count: usize, _level: ComplexityLevel) -> String {
        format!(
            r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Category:[id,name,parent,margin]
%S:Product:[id,name,sku,price,cost,stock,category]
%S:Customer:[id,name,email,tier,lifetime_value]
%S:Order:[id,customer,status,total,created_at]
%S:LineItem:[id,order,product,quantity,unit_price]
%N:Order>LineItem
---
categories:@Category
 |cat001,Electronics,~,0.25
 |cat002,Computers,@Category:cat001,0.20
 |cat003,Accessories,@Category:cat001,0.35
 |cat004,Home,~,0.40
products:@Product
{}
customers:@Customer
{}
orders:@Order
{}
"#,
            self.generate_ecommerce_products(count.min(50)),
            self.generate_ecommerce_customers(count.min(30)),
            self.generate_ecommerce_orders(count.min(20))
        )
    }

    fn generate_ecommerce_products(&self, count: usize) -> String {
        let mut lines = String::new();
        let names = [
            "Laptop Pro",
            "Wireless Mouse",
            "USB-C Hub",
            "Monitor 27in",
            "Keyboard",
            "Webcam HD",
        ];
        let categories = ["cat001", "cat002", "cat003", "cat004"];

        for i in 1..=count {
            let name = names[i % names.len()];
            let cat = categories[i % categories.len()];
            let price = 49.99 + (i as f64 * 23.45) % 1000.0;
            let cost = price * 0.65;
            let stock = 10 + (i * 7) % 500;

            lines.push_str(&format!(
                " |prod{:03},{},SKU{:05},{:.2},{:.2},{},@Category:{}\n",
                i, name, i, price, cost, stock, cat
            ));
        }
        lines
    }

    fn generate_ecommerce_customers(&self, count: usize) -> String {
        let mut lines = String::new();
        let names = [
            "Alice Johnson",
            "Bob Smith",
            "Carol Williams",
            "David Brown",
            "Eve Davis",
        ];
        let tiers = ["bronze", "silver", "gold", "platinum"];

        for i in 1..=count {
            let name = names[i % names.len()];
            let tier = tiers[i % tiers.len()];
            let ltv = 100.0 + (i as f64 * 234.56) % 10000.0;

            lines.push_str(&format!(
                " |cust{:03},{},{}.{}@email.com,{},{:.2}\n",
                i,
                name,
                // SAFETY: name is from hardcoded list of "Firstname Lastname" strings
                name.split_whitespace()
                    .next()
                    .expect("name has first word")
                    .to_lowercase(),
                i,
                tier,
                ltv
            ));
        }
        lines
    }

    fn generate_ecommerce_orders(&self, count: usize) -> String {
        let mut lines = String::new();
        let statuses = ["pending", "processing", "shipped", "delivered", "cancelled"];

        for i in 1..=count {
            let status = statuses[i % statuses.len()];
            let cust = 1 + (i % 10);
            let total = 50.0 + (i as f64 * 45.67) % 500.0;
            let day = 1 + (i % 28);
            let hour = i % 24;

            lines.push_str(&format!(
                " |ord{:03},@Customer:cust{:03},{},{:.2},2024-12-{:02}T{:02}:30:00Z\n",
                i, cust, status, total, day, hour
            ));

            // Add line items
            let item_count = 1 + (i % 3);
            for j in 1..=item_count {
                let prod = 1 + ((i + j) % 20);
                let qty = 1 + (j % 5);
                let price = 29.99 + (j as f64 * 10.0);
                lines.push_str(&format!(
                    " |li{:03}{},@Order:ord{:03},@Product:prod{:03},{},{:.2}\n",
                    i, j, i, prod, qty, price
                ));
            }
        }
        lines
    }

    fn generate_iot(&self, count: usize, _level: ComplexityLevel) -> String {
        format!(
            r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Device:[id,name,type,location,firmware,status]
%S:Sensor:[id,device,type,unit,min_threshold,max_threshold]
%S:Reading:[id,sensor,value,timestamp,quality]
%S:Alert:[id,sensor,type,severity,message,triggered_at,resolved_at]
%N:Device>Sensor
---
devices:@Device
{}
readings:@Reading
{}
alerts:@Alert
{}
"#,
            self.generate_iot_devices(count.min(20)),
            self.generate_iot_readings(count),
            self.generate_iot_alerts(count.min(10))
        )
    }

    fn generate_iot_devices(&self, count: usize) -> String {
        let mut lines = String::new();
        let types = ["gateway", "sensor_hub", "actuator", "monitor"];
        let locations = ["Building A", "Building B", "Warehouse", "Server Room"];
        let statuses = ["online", "offline", "maintenance", "error"];

        for i in 1..=count {
            let dev_type = types[i % types.len()];
            let location = locations[i % locations.len()];
            let status = statuses[i % statuses.len()];

            lines.push_str(&format!(
                " |dev{:03},Device-{:03},{},{},v2.1.{},{}\n",
                i,
                i,
                dev_type,
                location,
                i % 10,
                status
            ));

            // Add sensors
            let sensor_types = ["temperature", "humidity", "pressure", "motion"];
            let units = ["°C", "%", "hPa", "bool"];
            for j in 1..=2 {
                let st = sensor_types[(i + j) % sensor_types.len()];
                let unit = units[(i + j) % units.len()];
                lines.push_str(&format!(
                    " |sens{:03}{},@Device:dev{:03},{},{},0,100\n",
                    i, j, i, st, unit
                ));
            }
        }
        lines
    }

    fn generate_iot_readings(&self, count: usize) -> String {
        let mut lines = String::new();
        let qualities = ["good", "fair", "poor"];

        for i in 1..=count.min(100) {
            let sensor = 1 + (i % 20);
            let sensor_sub = 1 + (i % 2);
            let value = 20.0 + (i as f64 * 1.234) % 80.0;
            let quality = qualities[i % qualities.len()];
            let min = i % 60;
            let sec = i % 60;

            lines.push_str(&format!(
                " |read{:04},@Sensor:sens{:03}{},{:.2},2024-12-22T10:{:02}:{:02}Z,{}\n",
                i, sensor, sensor_sub, value, min, sec, quality
            ));
        }
        lines
    }

    fn generate_iot_alerts(&self, count: usize) -> String {
        let mut lines = String::new();
        let types = ["threshold_exceeded", "device_offline", "anomaly_detected"];
        let severities = ["info", "warning", "critical"];

        for i in 1..=count {
            let alert_type = types[i % types.len()];
            let severity = severities[i % severities.len()];
            let sensor = 1 + (i % 10);
            let resolved = if i % 3 == 0 {
                "~"
            } else {
                "2024-12-22T11:00:00Z"
            };

            lines.push_str(&format!(
                " |alert{:03},@Sensor:sens{:03}1,{},{},Alert message {},2024-12-22T10:30:00Z,{}\n",
                i, sensor, alert_type, severity, i, resolved
            ));
        }
        lines
    }

    fn generate_scientific(&self, count: usize, _level: ComplexityLevel) -> String {
        format!(
            r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Author:[id,name,affiliation,orcid,h_index]
%S:Publication:[id,title,authors,journal,year,citations,doi]
%S:Experiment:[id,title,hypothesis,methodology,status]
%S:Measurement:[id,experiment,variable,value,uncertainty,unit,timestamp]
%S:Dataset:[id,experiment,name,size_mb,format,checksum]
%N:Experiment>Measurement
---
authors:@Author
 |auth001,Dr. Marie Curie,Paris University,0000-0001-1234-5678,45
 |auth002,Dr. Albert Einstein,Princeton,0000-0001-2345-6789,120
 |auth003,Dr. Richard Feynman,Caltech,0000-0001-3456-7890,89
publications:@Publication
{}
experiments:@Experiment
{}
"#,
            self.generate_scientific_publications(count.min(20)),
            self.generate_scientific_experiments(count.min(10))
        )
    }

    fn generate_scientific_publications(&self, count: usize) -> String {
        let mut lines = String::new();
        let journals = [
            "Nature",
            "Science",
            "PNAS",
            "Physical Review",
            "Cell",
            "Lancet",
        ];
        let titles = [
            "Novel Findings in Quantum Mechanics",
            "Analysis of Complex Systems",
            "Machine Learning Applications",
            "Climate Change Patterns",
            "Genetic Markers Study",
        ];

        for i in 1..=count {
            let title = titles[i % titles.len()];
            let journal = journals[i % journals.len()];
            let year = 2020 + (i % 5);
            let citations = (i * 17) % 500;
            let auth_count = 1 + (i % 3);
            let authors: Vec<String> = (1..=auth_count)
                .map(|j| format!("@Author:auth{:03}", 1 + ((i + j) % 3)))
                .collect();

            let authors_str = authors.join(", ");
            lines.push_str(&format!(
                " |pub{:03},{},{},{},{},{},10.1234/pub{:03}\n",
                i,
                title,
                format_args!("[{authors_str}]"),
                journal,
                year,
                citations,
                i
            ));
        }
        lines
    }

    fn generate_scientific_experiments(&self, count: usize) -> String {
        let mut lines = String::new();
        let statuses = ["planned", "in_progress", "completed", "published"];
        let methodologies = [
            "randomized controlled trial",
            "observational study",
            "meta-analysis",
            "simulation",
        ];

        for i in 1..=count {
            let status = statuses[i % statuses.len()];
            let methodology = methodologies[i % methodologies.len()];

            lines.push_str(&format!(
                " |exp{:03},Experiment {:03},Testing hypothesis {},{},({})\n",
                i, i, i, methodology, status
            ));

            // Add measurements
            for j in 1..=5 {
                let value = 1.0 + (i as f64 * j as f64 * 0.123);
                let uncertainty = value * 0.05;
                let month = 1 + ((i + j) % 12);
                let day = 1 + (j % 28);

                lines.push_str(&format!(
                    " |meas{:03}{},@Experiment:exp{:03},variable_{},{:.4},{:.4},units,2024-{:02}-{:02}T12:00:00Z\n",
                    i, j, i, j, value, uncertainty, month, day
                ));
            }
        }
        lines
    }

    fn generate_enterprise(&self, count: usize, level: ComplexityLevel) -> String {
        let nesting = if level >= ComplexityLevel::L3Intermediate {
            "%N:Company>Division\n%N:Division>Department\n%N:Department>Team\n%N:Team>Employee"
        } else {
            ""
        };

        format!(
            r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Company:[id,name,industry,founded,headquarters]
%S:Division:[id,name,head,budget,headcount]
%S:Department:[id,name,division,budget,location]
%S:Team:[id,name,focus,lead,size]
%S:Employee:[id,name,role,salary,hire_date,status]
%S:Project:[id,name,owner,budget,status,deadline]
{}
---
companies:@Company
 |corp001,TechCorp Global,Technology,1995,San Francisco
{}
projects:@Project
{}
"#,
            nesting,
            self.generate_enterprise_structure(count, level),
            self.generate_enterprise_projects(count.min(15))
        )
    }

    fn generate_enterprise_structure(&self, count: usize, level: ComplexityLevel) -> String {
        let mut lines = String::new();
        let div_count = (count / 20).clamp(2, 5);
        let dept_per_div = (count / div_count / 5).clamp(2, 4);
        let team_per_dept = 2;

        let names = [
            "Alice Chen",
            "Bob Kumar",
            "Carol Smith",
            "David Lee",
            "Eve Wilson",
            "Frank Zhang",
            "Grace Park",
            "Henry Brown",
        ];
        let roles = [
            "VP",
            "Director",
            "Manager",
            "Lead",
            "Senior Engineer",
            "Engineer",
            "Analyst",
        ];
        let statuses = ["active", "on_leave", "remote"];

        let mut emp_id = 1;

        for d in 1..=div_count {
            let div_budget = 5000000 + (d * 1000000);
            let div_head = emp_id;
            lines.push_str(&format!(
                " |div{:03},Division {},@Employee:emp{:03},{},{}\n",
                d,
                ['A', 'B', 'C', 'D', 'E'][d - 1],
                div_head,
                div_budget,
                d * dept_per_div * team_per_dept * 3
            ));

            if level < ComplexityLevel::L3Intermediate {
                continue;
            }

            for dept in 1..=dept_per_div {
                let dept_num = (d - 1) * dept_per_div + dept;
                let dept_budget = 500000 + (dept_num * 100000);
                lines.push_str(&format!(
                    " |dept{:03},Department {},@Division:div{:03},{},Building {}\n",
                    dept_num,
                    dept_num,
                    d,
                    dept_budget,
                    ['A', 'B', 'C'][dept % 3]
                ));

                for team in 1..=team_per_dept {
                    let team_num = (dept_num - 1) * team_per_dept + team;
                    let focus = ["Frontend", "Backend", "Data", "Infra"][team_num % 4];
                    lines.push_str(&format!(
                        "   |team{:03},{} Team,{},@Employee:emp{:03},{}\n",
                        team_num, focus, focus, emp_id, 4
                    ));

                    // Add employees
                    for e in 1..=4 {
                        let name = names[emp_id % names.len()];
                        let role = roles[(emp_id + e) % roles.len()];
                        let salary = 80000 + ((emp_id * 5000) % 100000);
                        let status = statuses[emp_id % statuses.len()];
                        let month = 1 + (emp_id % 12);

                        lines.push_str(&format!(
                            "   |emp{:03},{},{},{},{},2020-{:02}-15\n",
                            emp_id, name, role, salary, status, month
                        ));
                        emp_id += 1;
                    }
                }
            }
        }
        lines
    }

    fn generate_enterprise_projects(&self, count: usize) -> String {
        let mut lines = String::new();
        let statuses = ["planning", "active", "on_hold", "completed"];
        let names = [
            "Platform Migration",
            "Customer Portal",
            "Data Pipeline",
            "Mobile App",
            "API Gateway",
        ];

        for i in 1..=count {
            let name = names[i % names.len()];
            let status = statuses[i % statuses.len()];
            let budget = 100000 + (i * 50000);
            let owner = 1 + (i % 20);
            let month = 1 + (i % 12);
            let day = 1 + (i % 28);

            lines.push_str(&format!(
                " |proj{:03},{} v{},@Employee:emp{:03},{},{},2025-{:02}-{:02}\n",
                i, name, i, owner, budget, status, month, day
            ));
        }
        lines
    }

    fn generate_social_media(&self, count: usize, _level: ComplexityLevel) -> String {
        format!(
            r#"%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,username,display_name,verified,followers,following,joined_at]
%S:Post:[id,author,content,likes,shares,comments,posted_at]
%S:Comment:[id,post,author,content,likes,replied_to]
%S:Follow:[id,follower,following,since]
%N:Post>Comment
---
users:@User
{}
posts:@Post
{}
follows:@Follow
{}
"#,
            self.generate_social_users(count.min(50)),
            self.generate_social_posts(count.min(30)),
            self.generate_social_follows(count.min(100))
        )
    }

    fn generate_social_users(&self, count: usize) -> String {
        let mut lines = String::new();
        let names = [
            "alice", "bob", "carol", "david", "eve", "frank", "grace", "henry",
        ];
        let display = [
            "Alice J", "Bob S", "Carol W", "David B", "Eve D", "Frank Z", "Grace P", "Henry B",
        ];

        for i in 1..=count {
            let username = format!("{}{}", names[i % names.len()], i);
            let display_name = format!("{} {}", display[i % display.len()], i);
            let verified = i % 5 == 0;
            let followers = (i * 123) % 10000;
            let following = (i * 47) % 1000;
            let month = 1 + (i % 12);

            lines.push_str(&format!(
                " |user{:03},{},{},{},{},{},2023-{:02}-01\n",
                i, username, display_name, verified, followers, following, month
            ));
        }
        lines
    }

    fn generate_social_posts(&self, count: usize) -> String {
        let mut lines = String::new();
        let contents = [
            "Just finished a great coding session!",
            "Check out this amazing sunset!",
            "Working on something exciting...",
            "Happy to announce...",
            "Question for my followers:",
        ];

        for i in 1..=count {
            let author = 1 + (i % 20);
            let content = contents[i % contents.len()];
            let likes = (i * 89) % 5000;
            let shares = likes / 10;
            let comment_count = (i * 7) % 50;
            let day = 1 + (i % 28);
            let hour = i % 24;

            lines.push_str(&format!(
                " |post{:03},@User:user{:03},{},{},{},{},2024-12-{:02}T{:02}:30:00Z\n",
                i, author, content, likes, shares, comment_count, day, hour
            ));

            // Add comments
            for j in 1..=(i % 4 + 1) {
                let commenter = 1 + ((i + j) % 30);
                let comment_likes = (j * 13) % 100;
                let replied = if j > 1 {
                    format!("@Comment:comm{:03}{}", i, j - 1)
                } else {
                    "~".to_string()
                };

                lines.push_str(&format!(
                    " |comm{:03}{},@Post:post{:03},@User:user{:03},Great post!,{},{}\n",
                    i, j, i, commenter, comment_likes, replied
                ));
            }
        }
        lines
    }

    fn generate_social_follows(&self, count: usize) -> String {
        let mut lines = String::new();

        for i in 1..=count {
            let follower = 1 + (i % 30);
            let following = 1 + ((i * 7) % 30);
            if follower != following {
                let month = 1 + (i % 12);
                lines.push_str(&format!(
                    " |follow{:03},@User:user{:03},@User:user{:03},2024-{:02}-01\n",
                    i, follower, following, month
                ));
            }
        }
        lines
    }
}

impl Default for DomainDatasetGenerator {
    fn default() -> Self {
        Self::new(42)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_domains() {
        assert_eq!(Domain::ALL.len(), 8);
        for domain in Domain::ALL {
            assert!(!domain.name().is_empty());
            assert!(!domain.code().is_empty());
            assert!(!domain.description().is_empty());
        }
    }

    #[test]
    fn test_domain_characteristics() {
        for domain in Domain::ALL {
            let chars = domain.characteristics();
            assert!(!chars.primary_types.is_empty());
            assert!(chars.typical_entity_count > 0);
        }
    }

    #[test]
    fn test_dataset_generation() {
        let generator = DomainDatasetGenerator::new(12345);

        for domain in Domain::ALL {
            let dataset = generator.generate(domain, ComplexityLevel::L3Intermediate);
            assert!(!dataset.hedl.is_empty());
            assert!(!dataset.id.is_empty());
            assert_eq!(dataset.domain, domain);
        }
    }

    #[test]
    fn test_finance_generation() {
        let generator = DomainDatasetGenerator::new(42);
        let dataset = generator.generate(Domain::Finance, ComplexityLevel::L4Advanced);

        assert!(dataset.hedl.contains("%S:Account"));
        assert!(dataset.hedl.contains("%S:Transaction"));
        assert!(dataset.hedl.contains("accounts:@Account"));
    }

    #[test]
    fn test_enterprise_nesting() {
        let generator = DomainDatasetGenerator::new(42);
        let dataset = generator.generate(Domain::Enterprise, ComplexityLevel::L4Advanced);

        assert!(dataset.hedl.contains("%N:Company>Division"));
        assert!(dataset.hedl.contains("%N:Division>Department"));
    }
}
