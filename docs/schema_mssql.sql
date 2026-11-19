
CREATE TABLE app_logs (
	id INTEGER NOT NULL IDENTITY, 
	timestamp DATETIME NULL, 
	level NVARCHAR(10) NOT NULL, 
	category NVARCHAR(50) NOT NULL, 
	action NVARCHAR(100) NOT NULL, 
	detail NTEXT NULL, 
	execution_id INTEGER NULL, 
	user_id INTEGER NULL, 
	value_sent NVARCHAR(200) NULL, 
	value_received NVARCHAR(200) NULL, 
	duration_ms INTEGER NULL, 
	success BIT NULL, 
	error_message NVARCHAR(500) NULL, 
	PRIMARY KEY (id)
)

;


CREATE TABLE configurations (
	id INTEGER NOT NULL IDENTITY, 
	config_key NVARCHAR(50) NOT NULL, 
	config_value NVARCHAR(500) NOT NULL, 
	updated_at DATETIME NULL, 
	PRIMARY KEY (id), 
	UNIQUE (config_key)
)

;


CREATE TABLE label_templates (
	id INTEGER NOT NULL IDENTITY, 
	name NVARCHAR(100) NOT NULL, 
	width_mm INTEGER NULL, 
	height_mm INTEGER NULL, 
	canvas_data NTEXT NULL, 
	is_default BIT NULL, 
	created_at DATETIME NULL, 
	updated_at DATETIME NULL, 
	PRIMARY KEY (id)
)

;


CREATE TABLE operators (
	id INTEGER NOT NULL IDENTITY, 
	name NVARCHAR(100) NOT NULL, 
	employee_id NVARCHAR(20) NOT NULL, 
	is_active BIT NULL, 
	created_at DATETIME NULL, 
	PRIMARY KEY (id), 
	UNIQUE (employee_id)
)

;


CREATE TABLE plc_tags (
	id INTEGER NOT NULL IDENTITY, 
	name NVARCHAR(50) NOT NULL, 
	db_number INTEGER NULL, 
	offset INTEGER NULL, 
	data_type NVARCHAR(10) NULL, 
	bit INTEGER NULL, 
	description NVARCHAR(200) NULL, 
	unit NVARCHAR(10) NULL, 
	PRIMARY KEY (id)
)

;


CREATE TABLE recipes (
	id INTEGER NOT NULL IDENTITY, 
	name NVARCHAR(100) NOT NULL, 
	model NVARCHAR(50) NOT NULL, 
	type NVARCHAR(50) NULL, 
	product NVARCHAR(100) NULL, 
	voltage_levels NTEXT NULL, 
	is_active BIT NULL, 
	created_at DATETIME NULL, 
	updated_at DATETIME NULL, 
	modified_by NVARCHAR(50) NULL, 
	PRIMARY KEY (id), 
	UNIQUE (model)
)

;


CREATE TABLE sync_log (
	id INTEGER NOT NULL IDENTITY, 
	direction NVARCHAR(10) NULL, 
	table_name NVARCHAR(50) NULL, 
	records_synced INTEGER NULL, 
	status NVARCHAR(20) NULL, 
	error_message NVARCHAR(500) NULL, 
	started_at DATETIME NULL, 
	completed_at DATETIME NULL, 
	PRIMARY KEY (id)
)

;


CREATE TABLE ui_state_log (
	id INTEGER NOT NULL IDENTITY, 
	timestamp DATETIME NULL, 
	execution_id INTEGER NULL, 
	current_step INTEGER NULL, 
	current_phase NVARCHAR(20) NULL, 
	background_color NVARCHAR(20) NULL, 
	message_displayed NVARCHAR(200) NULL, 
	PRIMARY KEY (id)
)

;


CREATE TABLE users (
	id INTEGER NOT NULL IDENTITY, 
	username NVARCHAR(50) NOT NULL, 
	password_hash NVARCHAR(255) NOT NULL, 
	role NVARCHAR(20) NULL, 
	is_active BIT NULL, 
	created_at DATETIME NULL, 
	PRIMARY KEY (id), 
	UNIQUE (username)
)

;


CREATE TABLE batches (
	id INTEGER NOT NULL IDENTITY, 
	batch_number NVARCHAR(50) NOT NULL, 
	operator_id INTEGER NULL, 
	scanned_at DATETIME NULL, 
	PRIMARY KEY (id), 
	UNIQUE (batch_number), 
	FOREIGN KEY(operator_id) REFERENCES operators (id)
)

;


CREATE TABLE import_export_logs (
	id INTEGER NOT NULL IDENTITY, 
	operation_type NVARCHAR(10) NOT NULL, 
	user_id INTEGER NULL, 
	file_name NVARCHAR(200) NULL, 
	file_size_bytes BIGINT NULL, 
	data_summary NTEXT NULL, 
	mode NVARCHAR(20) NULL, 
	conflicts_found INTEGER NULL, 
	records_imported INTEGER NULL, 
	status NVARCHAR(20) NULL, 
	error_message NVARCHAR(500) NULL, 
	performed_at DATETIME NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(user_id) REFERENCES users (id)
)

;


CREATE TABLE recipe_components (
	id INTEGER NOT NULL IDENTITY, 
	recipe_id INTEGER NOT NULL, 
	name NVARCHAR(50) NOT NULL, 
	voltage NVARCHAR(20) NOT NULL, 
	enabled BIT NULL, 
	min_value FLOAT NULL, 
	max_value FLOAT NULL, 
	unit NVARCHAR(10) NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(recipe_id) REFERENCES recipes (id)
)

;


CREATE TABLE recipe_totals (
	id INTEGER NOT NULL IDENTITY, 
	recipe_id INTEGER NOT NULL, 
	voltage NVARCHAR(20) NOT NULL, 
	min_total FLOAT NOT NULL, 
	max_total FLOAT NOT NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(recipe_id) REFERENCES recipes (id)
)

;


CREATE TABLE executions (
	id INTEGER NOT NULL IDENTITY, 
	recipe_id INTEGER NULL, 
	operator_id INTEGER NULL, 
	batch_id INTEGER NULL, 
	serial_number NVARCHAR(100) NULL, 
	plc_start_time DATETIME NULL, 
	plc_end_time DATETIME NULL, 
	plc_duration FLOAT NULL, 
	plc_pass_fail BIT NULL, 
	hipot_start_time DATETIME NULL, 
	hipot_end_time DATETIME NULL, 
	hipot_duration FLOAT NULL, 
	hipot_pass_fail BIT NULL, 
	final_status NVARCHAR(10) NULL, 
	total_duration FLOAT NULL, 
	synced_to_remote BIT NULL, 
	synced_at DATETIME NULL, 
	created_at DATETIME NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(recipe_id) REFERENCES recipes (id), 
	FOREIGN KEY(operator_id) REFERENCES operators (id), 
	FOREIGN KEY(batch_id) REFERENCES batches (id)
)

;


CREATE TABLE barcode_logs (
	id INTEGER NOT NULL IDENTITY, 
	code NVARCHAR(100) NOT NULL, 
	barcode_type NVARCHAR(20) NULL, 
	scan_order INTEGER NULL, 
	execution_id INTEGER NULL, 
	scanned_at DATETIME NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(execution_id) REFERENCES executions (id)
)

;


CREATE TABLE hipot_results (
	id INTEGER NOT NULL IDENTITY, 
	execution_id INTEGER NULL, 
	recipe_id INTEGER NULL, 
	voltage FLOAT NULL, 
	[current] FLOAT NULL, 
	resistance FLOAT NULL, 
	insulation_resistance FLOAT NULL, 
	test_duration FLOAT NULL, 
	is_pass BIT NULL, 
	timestamp DATETIME NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(execution_id) REFERENCES executions (id), 
	FOREIGN KEY(recipe_id) REFERENCES recipes (id)
)

;


CREATE TABLE label_prints (
	id INTEGER NOT NULL IDENTITY, 
	execution_id INTEGER NULL, 
	template_id INTEGER NULL, 
	printed_by NVARCHAR(50) NULL, 
	printed_at DATETIME NULL, 
	printer_model NVARCHAR(50) NULL, 
	connection_type NVARCHAR(10) NULL, 
	zpl_data NTEXT NULL, 
	status NVARCHAR(20) NULL, 
	error_message NVARCHAR(200) NULL, 
	copies INTEGER NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(execution_id) REFERENCES executions (id), 
	FOREIGN KEY(template_id) REFERENCES label_templates (id)
)

;


CREATE TABLE plc_readings (
	id INTEGER NOT NULL IDENTITY, 
	execution_id INTEGER NULL, 
	recipe_id INTEGER NULL, 
	component_name NVARCHAR(50) NULL, 
	voltage NVARCHAR(20) NULL, 
	tag_name NVARCHAR(50) NULL, 
	value FLOAT NULL, 
	min_limit FLOAT NULL, 
	max_limit FLOAT NULL, 
	is_pass BIT NULL, 
	timestamp DATETIME NULL, 
	PRIMARY KEY (id), 
	FOREIGN KEY(execution_id) REFERENCES executions (id), 
	FOREIGN KEY(recipe_id) REFERENCES recipes (id)
)

;
